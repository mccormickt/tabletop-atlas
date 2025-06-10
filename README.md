# Tabletop Atlas

A comprehensive application for board game enthusiasts to manage game rules, house rules, and ask questions about gameplay through an LLM-powered chat interface.

## Overview

Tabletop Atlas allows users to:
- Upload PDF exports of board game rules and store embeddings
- Add custom "House Rules" for each game to augment standard rules - Use an LLM chat interface to ask questions about game rules

## Development

### Storage

Currently uses in-memory HashMaps wrapped in Arc<Mutex<T>> for thread safety. This provides:
- Fast read/write access for games, house rules, and PDF document metadata
- Simple development setup
- Thread-safe concurrent access

**Note**:
- Data will be lost when the server restarts
- PDF content is stored as text strings (not actual files yet)
- This will be replaced with a database + file storage in future iterations
