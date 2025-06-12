use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dropshot::{
    ApiDescription, ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServerStarter,
};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

mod handlers;
mod models;
mod db;

use handlers::*;

pub struct AppState {
    db: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut db = Connection::open(path)?;
        
        // Run migrations
        let migrations = Migrations::new(vec![
            M::up(include_str!("../../migrations/V001__create_games_table.sql")),
            M::up(include_str!("../../migrations/V002__create_house_rules_table.sql")),
            M::up(include_str!("../../migrations/V003__create_embeddings_table.sql")),
        ]);
        
        migrations.to_latest(&mut db)?;
        
        Ok(Self { 
            db: Arc::new(Mutex::new(db)) 
        })
    }

    pub fn db(&self) -> Arc<Mutex<Connection>> {
        self.db.clone()
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
        default_request_body_max_bytes: 10 * 1024 * 1024, // 10MB for PDF uploads
        default_handler_task_mode: dropshot::HandlerTaskMode::Detached,
        log_headers: Default::default(),
    };

    // Create API description
    let mut api = ApiDescription::new();
    
    // Register API endpoints
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
    api.register(chat::chat_with_rules)?;

    let app_state = AppState::new("atlas.db")?;
    let server = HttpServerStarter::new(&config_dropshot, api, app_state, &log)
        .map_err(|error| format!("failed to create server: {}", error))?
        .start();

    println!("🎲 Tabletop Atlas Server running on http://127.0.0.1:8080");
    server.await?;
    Ok(())
}