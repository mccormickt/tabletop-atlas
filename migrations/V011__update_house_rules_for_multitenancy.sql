ALTER TABLE house_rules ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE house_rules ADD COLUMN game_type TEXT DEFAULT 'master' CHECK (game_type IN ('master', 'custom'));
ALTER TABLE house_rules ADD COLUMN is_public INTEGER DEFAULT 0;

CREATE INDEX idx_house_rules_user_id ON house_rules(user_id);
CREATE INDEX idx_house_rules_game_type ON house_rules(game_type);
