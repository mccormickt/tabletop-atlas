use std::path::Path;

use anyhow::Result;
use clap::{Arg, Command};
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter,
};
use rusqlite::{Connection, ffi::sqlite3_auto_extension};
use rusqlite_migration::{M, Migrations};
use sqlite_vec::sqlite3_vec_init;

mod agents;
mod auth;
mod bgg;
mod config;
mod db;
mod embeddings;
mod error;
mod handlers;
mod models;
mod pdf;
mod tools;

use agents::RulesChatAgent;
use config::OllamaConfig;
use db::Database;
use embeddings::Embedder;
use handlers::static_files;
use handlers::*;
use rig::prelude::{CompletionClient as _, EmbeddingsClient};
use rig::providers::ollama;

const NOMIC_EMBED_DIMS: usize = 768;

pub struct AppState {
    db: Database,
    embeddings: Embedder<ollama::EmbeddingModel>,
    rules_agent: RulesChatAgent<ollama::CompletionModel>,
}

impl AppState {
    pub fn new(path: impl AsRef<Path>, ollama_config: &OllamaConfig) -> Result<Self> {
        // Initialize sqlite-vec extension
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut std::os::raw::c_char,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> std::os::raw::c_int,
            >(sqlite3_vec_init as *const ())));
        }

        let mut db = Connection::open(path)?;

        // Run migrations
        let migrations = Migrations::new(vec![
            M::up(include_str!(
                "../../migrations/V001__create_games_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V002__create_house_rules_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V003__create_embeddings_table.sql"
            )),
            M::up(include_str!("../../migrations/V004__seed_games_data.sql")),
            M::up(include_str!(
                "../../migrations/V005__add_house_rules_toggle_to_chat_sessions.sql"
            )),
            M::up(include_str!(
                "../../migrations/V006__create_users_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V007__create_sessions_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V008__rename_games_to_master_games.sql"
            )),
            M::up(include_str!(
                "../../migrations/V009__create_user_collections_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V010__create_custom_games_table.sql"
            )),
            M::up(include_str!(
                "../../migrations/V011__update_house_rules_for_multitenancy.sql"
            )),
            M::up(include_str!(
                "../../migrations/V012__update_embeddings_for_multitenancy.sql"
            )),
            M::up(include_str!(
                "../../migrations/V013__update_chat_sessions_for_multitenancy.sql"
            )),
            M::up(include_str!(
                "../../migrations/V014__create_challenges_tables.sql"
            )),
        ]);

        migrations.to_latest(&mut db)?;

        // Build a single shared Ollama client for all LLM/embedding services
        let client = ollama_config.build_client()?;

        // Create models from the client (client is borrowed, not moved)
        let embedding_model =
            client.embedding_model_with_ndims(&ollama_config.embedding_model, NOMIC_EMBED_DIMS);
        let completion_model = client.completion_model(&ollama_config.llm_model);

        Ok(Self {
            db: Database::new(db),
            embeddings: Embedder::new(embedding_model, ollama_config.embedding_model.clone()),
            rules_agent: RulesChatAgent::new(completion_model),
        })
    }

    pub fn db(&self) -> Database {
        self.db.clone()
    }

    pub fn embedder(&self) -> &Embedder<ollama::EmbeddingModel> {
        &self.embeddings
    }

    pub fn rules_agent(&self) -> &RulesChatAgent<ollama::CompletionModel> {
        &self.rules_agent
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file early so all config readers see the values
    dotenvy::dotenv().ok();

    let matches = Command::new("tabletop-atlas-backend")
        .version("0.1.0")
        .author("Tabletop Atlas Team")
        .about("Backend server for Tabletop Atlas - a board game management application")
        .arg(
            Arg::new("openapi")
                .long("openapi")
                .help("Generate OpenAPI specification and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bind-address")
                .short('a')
                .long("bind-address")
                .help("Address to bind the server to")
                .value_name("ADDRESS")
                .conflicts_with("port"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Port to bind the server to (binds to 127.0.0.1)")
                .value_name("PORT"),
        )
        .arg(
            Arg::new("ollama-url")
                .short('u')
                .long("ollama-url")
                .env("OLLAMA_URL")
                .default_value("http://localhost:11434")
                .help("Ollama API base URL")
                .value_name("URL"),
        )
        .arg(
            Arg::new("llm-model")
                .long("llm-model")
                .env("LLM_MODEL")
                .default_value("gpt-oss:latest")
                .help("Chat completion model name")
                .value_name("MODEL"),
        )
        .arg(
            Arg::new("embedding-model")
                .long("embedding-model")
                .env("EMBEDDING_MODEL")
                .default_value("nomic-embed-text:latest")
                .help("Embedding model name")
                .value_name("MODEL"),
        )
        // Auth configuration
        .arg(
            Arg::new("google-client-id")
                .long("google-client-id")
                .env("GOOGLE_CLIENT_ID")
                .help("Google OAuth client ID")
                .value_name("ID")
                .required_unless_present("openapi"),
        )
        .arg(
            Arg::new("google-client-secret")
                .long("google-client-secret")
                .env("GOOGLE_CLIENT_SECRET")
                .hide_env_values(true)
                .help("Google OAuth client secret")
                .value_name("SECRET")
                .required_unless_present("openapi"),
        )
        .arg(
            Arg::new("google-redirect-uri")
                .long("google-redirect-uri")
                .env("GOOGLE_REDIRECT_URI")
                .default_value("http://localhost:8080/api/auth/callback")
                .help("Google OAuth redirect URI")
                .value_name("URI"),
        )
        .arg(
            Arg::new("jwt-secret")
                .long("jwt-secret")
                .env("JWT_SECRET")
                .hide_env_values(true)
                .help("Secret key for signing JWT tokens")
                .value_name("SECRET")
                .required_unless_present("openapi"),
        )
        .arg(
            Arg::new("jwt-access-expiry")
                .long("jwt-access-expiry")
                .env("JWT_ACCESS_EXPIRY")
                .default_value("900")
                .help("JWT access token expiry in seconds")
                .value_name("SECONDS"),
        )
        .arg(
            Arg::new("jwt-refresh-expiry")
                .long("jwt-refresh-expiry")
                .env("JWT_REFRESH_EXPIRY")
                .default_value("604800")
                .help("JWT refresh token expiry in seconds")
                .value_name("SECONDS"),
        )
        .arg(
            Arg::new("frontend-url")
                .long("frontend-url")
                .env("FRONTEND_URL")
                .default_value("http://localhost:8080")
                .help("Frontend URL for redirects after auth")
                .value_name("URL"),
        )
        // BGG configuration
        .arg(
            Arg::new("bgg-api-token")
                .long("bgg-api-token")
                .env("BGG_API_TOKEN")
                .hide_env_values(true)
                .help("BoardGameGeek API token for authenticated requests")
                .value_name("TOKEN"),
        )
        .get_matches();

    // Check if --openapi flag is provided
    if matches.get_flag("openapi") {
        generate_openapi().await?;
        return Ok(());
    }

    // Initialize auth configuration from clap-resolved values
    let auth_config = auth::AuthConfig::from_matches(&matches);
    auth::AuthConfig::init_with(auth_config)
        .map_err(|e| format!("Failed to init auth config: {}", e))?;
    auth::OidcClient::init().map_err(|e| format!("Failed to init OIDC client: {}", e))?;

    // Build Ollama/LLM config (clap resolves CLI arg > env var > default)
    let ollama_config = OllamaConfig::from_matches(&matches);

    // Determine bind address with priority: --bind-address > --port > PORT env var > default
    let bind_address = if let Some(addr) = matches.get_one::<String>("bind-address") {
        addr.clone()
    } else if let Some(port) = matches.get_one::<String>("port") {
        format!("127.0.0.1:{}", port)
    } else if let Ok(port) = std::env::var("PORT") {
        format!("127.0.0.1:{}", port)
    } else {
        "127.0.0.1:8080".to_string()
    };

    // Set up logging
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("tabletop-atlas")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Set up the server
    let config_dropshot = ConfigDropshot {
        bind_address: bind_address.parse()?,
        default_request_body_max_bytes: 15 * 1024 * 1024, // 15MB for PDF/CSV uploads
        default_handler_task_mode: dropshot::HandlerTaskMode::Detached,
        log_headers: Default::default(),
    };

    // Create API description
    let api = create_api_description()?;

    let app_state = AppState::new("atlas.db", &ollama_config)?;

    // Best-effort connectivity check — server starts regardless, but log reflects reality
    match app_state.embedder().test_connection().await {
        Ok(()) => slog::info!(log, "Ollama API connected";
            "api_base" => &ollama_config.api_base,
            "llm_model" => &ollama_config.llm_model,
            "embedding_model" => &ollama_config.embedding_model,
        ),
        Err(e) => slog::warn!(log, "Ollama API configured but not reachable";
            "api_base" => &ollama_config.api_base,
            "llm_model" => &ollama_config.llm_model,
            "embedding_model" => &ollama_config.embedding_model,
            "error" => %e,
        ),
    }

    let server = HttpServerStarter::new(&config_dropshot, api, app_state, &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();
    server.await?;
    Ok(())
}

fn create_api_description() -> Result<ApiDescription<AppState>, Box<dyn std::error::Error>> {
    let mut api = ApiDescription::new();

    // Register API endpoints first (these have higher priority)
    api.register(games::list_games)?;
    api.register(games::get_game)?;
    api.register(games::create_game)?;
    api.register(games::update_game)?;
    api.register(games::delete_game)?;

    api.register(house_rules::list_house_rules)?;
    api.register(house_rules::get_house_rule)?;
    api.register(house_rules::create_house_rule)?;
    api.register(house_rules::update_house_rule)?;
    api.register(house_rules::delete_house_rule)?;

    api.register(upload::upload_rules_pdf)?;
    api.register(upload::get_rules_info)?;
    api.register(upload::delete_rules)?;
    api.register(chat::chat_with_rules)?;
    api.register(chat::list_chat_sessions)?;
    api.register(chat::get_chat_session)?;
    api.register(chat::create_chat_session)?;
    api.register(chat::update_chat_session)?;
    api.register(chat::search_rules)?;

    // Register auth endpoints
    api.register(handlers::auth::login)?;
    api.register(handlers::auth::callback)?;
    api.register(handlers::auth::get_me)?;
    api.register(handlers::auth::logout)?;
    api.register(handlers::auth::refresh)?;

    // Register collection endpoints
    api.register(collections::list_collection)?;
    api.register(collections::add_to_collection)?;
    api.register(collections::update_collection_entry)?;
    api.register(collections::remove_from_collection)?;

    // Register custom games endpoints
    api.register(custom_games::list_custom_games)?;
    api.register(custom_games::list_public_custom_games)?;
    api.register(custom_games::create_custom_game)?;
    api.register(custom_games::get_custom_game)?;
    api.register(custom_games::update_custom_game)?;
    api.register(custom_games::delete_custom_game)?;

    // Register challenge endpoints
    api.register(challenges::list_challenges)?;
    api.register(challenges::create_challenge)?;
    api.register(challenges::get_challenge)?;
    api.register(challenges::update_challenge)?;
    api.register(challenges::delete_challenge)?;
    api.register(challenges::get_challenge_grid)?;
    api.register(challenges::add_participant)?;
    api.register(challenges::remove_participant)?;
    api.register(challenges::assign_game)?;
    api.register(challenges::remove_game)?;
    api.register(challenges::record_play)?;
    api.register(challenges::update_play)?;
    api.register(challenges::delete_play)?;
    api.register(challenges::get_challenge_stats)?;

    // Register tools endpoints
    api.register(handlers::tools::list_tools)?;
    api.register(handlers::tools::get_tool)?;
    api.register(handlers::tools::calculate_scores)?;

    // Register admin endpoints
    api.register(handlers::admin::get_admin_stats)?;
    api.register(handlers::admin::list_admin_users)?;
    api.register(handlers::admin::update_user_role)?;
    api.register(handlers::admin::preview_bgg_import)?;
    api.register(handlers::admin::execute_bgg_import)?;
    api.register(handlers::admin::get_enrichment_stats)?;
    api.register(handlers::admin::preview_bgg_enrich)?;
    api.register(handlers::admin::execute_bgg_enrich)?;
    api.register(handlers::admin::preview_bulk_enrich)?;
    api.register(handlers::admin::execute_bulk_enrich)?;

    // Register health check
    api.register(static_files::health_check)?;

    // Register specific static file handlers
    api.register(static_files::serve_favicon)?;
    api.register(static_files::serve_app_assets)?;

    // Register specific SPA routes
    api.register(static_files::serve_games_views)?; // /games/{path:.*}
    api.register(static_files::serve_search_view)?; // /search
    api.register(static_files::serve_upload_view)?; // /upload
    api.register(static_files::serve_chat_view)?; // /chat
    api.register(static_files::serve_collection_view)?; // /collection
    api.register(static_files::serve_auth_views)?; // /auth/{path:.*}
    api.register(static_files::serve_challenges_views)?; // /challenges/{path:.*}
    api.register(static_files::serve_tools_views)?; // /tools/{path:.*}
    api.register(static_files::serve_admin_views)?; // /admin/{path:.*}
    api.register(static_files::serve_index)?; // /

    Ok(api)
}

async fn generate_openapi() -> Result<(), Box<dyn std::error::Error>> {
    let api = create_api_description()?;

    let mut openapi = api.openapi("Tabletop Atlas API", semver::Version::new(1, 0, 0));

    openapi
        .description("API for managing board games, house rules, and AI-powered chat")
        .contact_url("https://github.com/mccormickt/tabletop-atlas")
        .license_name("MIT");

    let json = openapi.json()?;
    println!("{}", json);

    Ok(())
}
