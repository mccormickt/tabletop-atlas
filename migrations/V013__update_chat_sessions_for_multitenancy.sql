ALTER TABLE chat_sessions ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE chat_sessions ADD COLUMN game_type TEXT DEFAULT 'master' CHECK (game_type IN ('master', 'custom'));

CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_game_type ON chat_sessions(game_type);
