use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{Game, HouseRule, PdfDocument};

/// In-memory storage for the application
#[derive(Clone)]
pub struct AppState {
    pub games: Arc<Mutex<HashMap<Uuid, Game>>>,
    pub house_rules: Arc<Mutex<HashMap<Uuid, HouseRule>>>,
    pub pdf_documents: Arc<Mutex<HashMap<Uuid, PdfDocument>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            games: Arc::new(Mutex::new(HashMap::new())),
            house_rules: Arc::new(Mutex::new(HashMap::new())),
            pdf_documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a game exists
    pub fn game_exists(&self, game_id: &Uuid) -> bool {
        let games = self.games.lock().unwrap();
        games.contains_key(game_id)
    }

    /// Get a game by ID
    pub fn get_game(&self, game_id: &Uuid) -> Option<Game> {
        let games = self.games.lock().unwrap();
        games.get(game_id).cloned()
    }

    /// Get all games
    pub fn get_all_games(&self) -> Vec<Game> {
        let games = self.games.lock().unwrap();
        games.values().cloned().collect()
    }

    /// Add a game
    pub fn add_game(&self, game: Game) {
        let mut games = self.games.lock().unwrap();
        games.insert(game.id, game);
    }

    /// Get all house rules for a specific game
    pub fn get_house_rules_for_game(&self, game_id: &Uuid) -> Vec<HouseRule> {
        let house_rules = self.house_rules.lock().unwrap();
        house_rules
            .values()
            .filter(|rule| rule.game_id == *game_id)
            .cloned()
            .collect()
    }

    /// Add a house rule
    pub fn add_house_rule(&self, house_rule: HouseRule) {
        let mut house_rules = self.house_rules.lock().unwrap();
        house_rules.insert(house_rule.id, house_rule);
    }

    /// Get all PDF documents for a specific game
    pub fn get_pdf_documents_for_game(&self, game_id: &Uuid) -> Vec<PdfDocument> {
        let pdf_documents = self.pdf_documents.lock().unwrap();
        pdf_documents
            .values()
            .filter(|doc| doc.game_id == *game_id)
            .cloned()
            .collect()
    }

    /// Add a PDF document
    pub fn add_pdf_document(&self, pdf_document: PdfDocument) {
        let mut pdf_documents = self.pdf_documents.lock().unwrap();
        pdf_documents.insert(pdf_document.id, pdf_document);
    }

    /// Get a PDF document by ID
    pub fn get_pdf_document(&self, document_id: &Uuid) -> Option<PdfDocument> {
        let pdf_documents = self.pdf_documents.lock().unwrap();
        pdf_documents.get(document_id).cloned()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
