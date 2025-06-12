use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter,
};
use rusqlite::Connection;

pub struct AppContext {
    db: Arc<Mutex<Connection>>,
}

impl AppContext {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = Connection::open(path)?;
        Ok(Self { db: Mutex::new(db) })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("tabletop-atlas")
        .map_err(|error| format!("failed to create logger: {}", error))?;

    // Set up the server
    let config_dropshot = ConfigDropshot {
        bind_address: "127.0.0.1:8080".parse()?,
        default_request_body_max_bytes: 1024 * 1024, // 1MB
        default_handler_task_mode: dropshot::HandlerTaskMode::Detached,
        log_headers: Default::default(),
    };

    // Create API description
    let mut api = ApiDescription::new();

    let app_state = AppState::new("atlas.db");
    let server = HttpServerStarter::new(&config_dropshot, api, app_state, &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    println!("🎲 Tabletop Atlas Server");
    server.await?;
    Ok(())
}
