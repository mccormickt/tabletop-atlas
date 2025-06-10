use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter,
};

use tabletop_atlas::handlers::*;
use tabletop_atlas::models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("tabletop-atlas")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Create API description
    let mut api = ApiDescription::new();
    api.register(create_game)?;
    api.register(list_games)?;
    api.register(get_game)?;
    api.register(upload_pdf_document)?;
    api.register(list_pdf_documents_for_game)?;
    api.register(create_house_rule)?;
    api.register(list_house_rules_for_game)?;

    // Set up the server
    let config_dropshot = ConfigDropshot {
        bind_address: "127.0.0.1:8080".parse()?,
        request_body_max_bytes: 1024 * 1024, // 1MB
        default_handler_task_mode: dropshot::HandlerTaskMode::Detached,
        log_headers: Default::default(),
    };

    let app_state = AppState::new();
    let server = HttpServerStarter::new(&config_dropshot, api, app_state, &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    println!("🎲 Tabletop Atlas Server");
    server.await?;
    Ok(())
}
