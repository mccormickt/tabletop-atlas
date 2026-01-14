CREATE TABLE custom_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    publisher TEXT,
    year_published INTEGER,
    min_players INTEGER,
    max_players INTEGER,
    play_time_minutes INTEGER,
    complexity_rating REAL CHECK (complexity_rating >= 1.0 AND complexity_rating <= 5.0),
    rules_pdf_path TEXT,
    rules_text TEXT,
    is_public INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_custom_games_user_id ON custom_games(user_id);
CREATE INDEX idx_custom_games_is_public ON custom_games(is_public);

CREATE TRIGGER update_custom_games_updated_at
    AFTER UPDATE ON custom_games FOR EACH ROW
BEGIN
    UPDATE custom_games SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
