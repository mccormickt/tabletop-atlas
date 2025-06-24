-- Create games table to store board game information
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    publisher TEXT,
    year_published INTEGER,
    min_players INTEGER,
    max_players INTEGER,
    play_time_minutes INTEGER,
    complexity_rating REAL CHECK (complexity_rating >= 1.0 AND complexity_rating <= 5.0),
    bgg_id INTEGER UNIQUE, -- BoardGameGeek ID for external reference
    rules_pdf_path TEXT, -- Path to uploaded PDF file
    rules_text TEXT, -- Extracted text from PDF for searching
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for searching games by name
CREATE INDEX idx_games_name ON games(name);

-- Index for BoardGameGeek ID lookups
CREATE INDEX idx_games_bgg_id ON games(bgg_id);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_games_updated_at 
    AFTER UPDATE ON games 
    FOR EACH ROW 
BEGIN
    UPDATE games SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;