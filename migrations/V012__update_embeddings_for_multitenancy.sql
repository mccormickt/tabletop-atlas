ALTER TABLE embeddings ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE embeddings ADD COLUMN game_type TEXT DEFAULT 'master' CHECK (game_type IN ('master', 'custom'));

CREATE INDEX idx_embeddings_user_id ON embeddings(user_id);
CREATE INDEX idx_embeddings_game_type ON embeddings(game_type);
