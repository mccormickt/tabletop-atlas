use std::path::Path;

use anyhow::Result;
use clap::{Arg, Command};
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter,
};
use rusqlite::{Connection, ffi::sqlite3_auto_extension};
use rusqlite_migration::{M, Migrations};
use sqlite_vec::sqlite3_vec_init;

mod db;
mod embeddings;
mod handlers;
mod llm;
mod models;
mod pdf;

use db::Database;
use embeddings::Embedder;
use handlers::static_files;
use handlers::*;
use llm::LLMClient;

pub struct AppState {
    db: Database,
    embeddings: Embedder,
    llm: LLMClient,
}

impl AppState {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        // Initialize sqlite-vec extension
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
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
        ]);

        migrations.to_latest(&mut db)?;

        Ok(Self {
            db: Database::new(db),
            embeddings: Embedder::new(),
            llm: LLMClient::new(),
        })
    }

    pub fn db(&self) -> Database {
        self.db.clone()
    }

    pub fn embedder(&self) -> &Embedder {
        &self.embeddings
    }

    pub fn llm(&self) -> &LLMClient {
        &self.llm
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                .default_value("127.0.0.1:8080"),
        )
        .get_matches();

    // Check if --openapi flag is provided
    if matches.get_flag("openapi") {
        generate_openapi().await?;
        return Ok(());
    }

    let bind_address = matches.get_one::<String>("bind-address").unwrap();

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
        default_request_body_max_bytes: 10 * 1024 * 1024, // 10MB for PDF uploads
        default_handler_task_mode: dropshot::HandlerTaskMode::Detached,
        log_headers: Default::default(),
    };

    // Create API description
    let api = create_api_description()?;

    let app_state = AppState::new("atlas.db")?;
    let server = HttpServerStarter::new(&config_dropshot, api, app_state, &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    println!("🎲 Tabletop Atlas Server running on {}", bind_address);
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
    api.register(chat::search_rules)?;

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
