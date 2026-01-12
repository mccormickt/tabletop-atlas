-- Main challenge entity
CREATE TABLE challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    owner_id INTEGER NOT NULL,
    grid_rows INTEGER NOT NULL DEFAULT 8 CHECK (grid_rows >= 1 AND grid_rows <= 10),
    grid_cols INTEGER NOT NULL DEFAULT 8 CHECK (grid_cols >= 1 AND grid_cols <= 10),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'completed', 'archived')),
    start_date DATE,
    end_date DATE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Challenge participants
CREATE TABLE challenge_participants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    role TEXT NOT NULL DEFAULT 'participant' CHECK (role IN ('owner', 'participant')),
    joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (challenge_id) REFERENCES challenges(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(challenge_id, user_id)
);

-- Grid rows (assigned games)
CREATE TABLE challenge_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_id INTEGER NOT NULL,
    row_index INTEGER NOT NULL CHECK (row_index >= 0),
    game_type TEXT NOT NULL CHECK (game_type IN ('master', 'custom', 'collection')),
    game_id INTEGER NOT NULL,
    display_name TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (challenge_id) REFERENCES challenges(id) ON DELETE CASCADE,
    UNIQUE(challenge_id, row_index)
);

-- Grid cells (play sessions)
CREATE TABLE challenge_plays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_id INTEGER NOT NULL,
    challenge_game_id INTEGER NOT NULL,
    col_index INTEGER NOT NULL CHECK (col_index >= 0),
    played_at DATE NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (challenge_id) REFERENCES challenges(id) ON DELETE CASCADE,
    FOREIGN KEY (challenge_game_id) REFERENCES challenge_games(id) ON DELETE CASCADE,
    UNIQUE(challenge_game_id, col_index)
);

-- Play participants and winners
CREATE TABLE challenge_play_participants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    challenge_play_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    is_winner BOOLEAN NOT NULL DEFAULT FALSE,
    score INTEGER,
    FOREIGN KEY (challenge_play_id) REFERENCES challenge_plays(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(challenge_play_id, user_id)
);

-- Performance indexes
CREATE INDEX idx_challenges_owner ON challenges(owner_id);
CREATE INDEX idx_challenges_status ON challenges(status);
CREATE INDEX idx_challenge_participants_challenge ON challenge_participants(challenge_id);
CREATE INDEX idx_challenge_participants_user ON challenge_participants(user_id);
CREATE INDEX idx_challenge_games_challenge ON challenge_games(challenge_id);
CREATE INDEX idx_challenge_plays_game ON challenge_plays(challenge_game_id);
CREATE INDEX idx_challenge_play_participants_play ON challenge_play_participants(challenge_play_id);

-- Update triggers
CREATE TRIGGER update_challenges_updated_at
    AFTER UPDATE ON challenges FOR EACH ROW
BEGIN UPDATE challenges SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

CREATE TRIGGER update_challenge_plays_updated_at
    AFTER UPDATE ON challenge_plays FOR EACH ROW
BEGIN UPDATE challenge_plays SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;
