CREATE TABLE user_collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    master_game_id INTEGER NOT NULL,
    notes TEXT,
    rating INTEGER CHECK (rating >= 1 AND rating <= 10),
    play_count INTEGER DEFAULT 0,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (master_game_id) REFERENCES master_games(id) ON DELETE CASCADE,
    UNIQUE(user_id, master_game_id)
);

CREATE INDEX idx_user_collections_user_id ON user_collections(user_id);
