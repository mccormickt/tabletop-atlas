-- Create house rules table to store custom rules for games
CREATE TABLE house_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT, -- e.g., "Setup", "Gameplay", "Scoring", "Variants"
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Index for finding house rules by game
CREATE INDEX idx_house_rules_game_id ON house_rules(game_id);

-- Index for finding active house rules
CREATE INDEX idx_house_rules_active ON house_rules(is_active);

-- Index for searching by category
CREATE INDEX idx_house_rules_category ON house_rules(category);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_house_rules_updated_at 
    AFTER UPDATE ON house_rules 
    FOR EACH ROW 
BEGIN
    UPDATE house_rules SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;