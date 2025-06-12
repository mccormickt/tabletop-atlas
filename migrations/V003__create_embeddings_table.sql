-- Create embeddings table to store vector embeddings for rule text chunks
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding BLOB NOT NULL, -- Serialized vector embedding
    chunk_index INTEGER NOT NULL, -- Order of chunk within the document
    source_type TEXT NOT NULL CHECK (source_type IN ('rules_pdf', 'house_rule')),
    source_id INTEGER, -- ID of house rule if source_type is 'house_rule'
    metadata TEXT, -- JSON metadata about the chunk (page number, section, etc.)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (source_id) REFERENCES house_rules(id) ON DELETE CASCADE
);

-- Index for finding embeddings by game
CREATE INDEX idx_embeddings_game_id ON embeddings(game_id);

-- Index for finding embeddings by source type
CREATE INDEX idx_embeddings_source_type ON embeddings(source_type);

-- Index for finding embeddings by source ID (for house rules)
CREATE INDEX idx_embeddings_source_id ON embeddings(source_id);

-- Index for chunk ordering
CREATE INDEX idx_embeddings_chunk_index ON embeddings(game_id, chunk_index);

-- Create chat sessions table to store conversation history
CREATE TABLE chat_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    title TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Create chat messages table to store individual messages
CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    context_chunks TEXT, -- JSON array of relevant embedding IDs used for this response
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);

-- Indexes for chat functionality
CREATE INDEX idx_chat_sessions_game_id ON chat_sessions(game_id);
CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at);

-- Trigger to update chat sessions updated_at timestamp
CREATE TRIGGER update_chat_sessions_updated_at 
    AFTER INSERT ON chat_messages 
    FOR EACH ROW 
BEGIN
    UPDATE chat_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.session_id;
END;