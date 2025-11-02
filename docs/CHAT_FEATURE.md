# Chat Feature Documentation

The Tabletop Atlas chat feature provides an AI-powered assistant that can answer questions about board game rules using the uploaded PDF rulebooks and embeddings-based context retrieval.

## Overview

The chat system consists of:
- **Backend**: Rust-based API with LLM integration (Ollama + Mistral)
- **Frontend**: Svelte-based chat interface
- **AI Pipeline**: Embedding-based context retrieval + LLM response generation

## Prerequisites

Before using the chat feature, ensure you have:

1. **Ollama running locally** on `http://localhost:11434`
2. **Mistral model downloaded** (the system uses `mistral-small:22b` by default)
3. **Games with uploaded PDF rules** (for embeddings)

### Setting up Ollama

1. Install Ollama from https://ollama.ai/
2. Pull the required models:
   ```bash
   ollama pull mistral-small3.2:24b
   ollama pull nomic-embed-text:latest
   ```
3. Verify Ollama is running:
   ```bash
   curl http://localhost:11434/api/tags
   ```

## How It Works

### 1. Document Processing
When a PDF is uploaded for a game:
- Text is extracted and chunked
- Each chunk is converted to embeddings using `nomic-embed-text`
- Embeddings are stored in SQLite with sqlite-vec for similarity search

### 2. Chat Pipeline
When a user asks a question:
1. **User message saved** to chat session
2. **Query embedding generated** for the user's question
3. **Similarity search** performed against rule chunks
4. **Context preparation** with top 5 most relevant chunks
5. **LLM prompt** created with context and user question
6. **AI response generated** using Mistral model
7. **Response saved** with context source references

### 3. Context Sources
Each AI response includes:
- **Similarity scores** for retrieved chunks
- **Source metadata** (page numbers, sections)
- **Chunk references** for transparency

## Using the Chat Interface

### Accessing Chat
1. Navigate to `/chat` in the application
2. Select a game that has uploaded PDF rules
3. Create a new chat session or continue an existing one

### Chat Sessions
- **Persistent conversations** with message history
- **Multiple sessions** per game for different topics
- **Automatic titles** based on game name
- **Session management** with timestamps and message counts

### Asking Questions
The AI works best with:
- **Specific rule questions**: "How do I resolve combat?"
- **Gameplay scenarios**: "What happens when I run out of cards?"
- **Turn structure**: "What's the order of phases in a turn?"
- **Win conditions**: "How do I win the game?"

### Example Interactions
```
User: "How do I win in Settlers of Catan?"
AI: "To win Settlers of Catan, you need to be the first player to reach 10 victory points. Victory points can be earned through: building settlements (1 point each), building cities (2 points each), having the longest road (2 points), having the largest army (2 points), and development cards (some are worth 1 point each)."

Context Sources:
- Chunk 15: "Victory conditions and point calculation" (92% similarity)
- Chunk 8: "Building settlements and cities" (87% similarity)
```

## API Endpoints

### Chat Sessions
- `GET /api/chat/sessions?gameId={id}` - List sessions for a game
- `POST /api/chat/sessions` - Create new session
- `GET /api/chat/sessions/{id}` - Get session with message history

### Messaging
- `POST /api/chat/message` - Send message and get AI response

### Rules Search
- `GET /api/chat/search-rules` - Direct similarity search in rules

## Configuration

### LLM Settings
The system uses these default settings for Mistral:
- **Temperature**: 0.7 (balanced creativity/consistency)
- **Max tokens**: 512 (reasonable response length)
- **Model**: `mistral-small3.2:24b`

### Embedding Settings
- **Model**: `nomic-embed-text:latest`
- **Similarity threshold**: 0.6 for chat context
- **Context limit**: Top 5 most relevant chunks
- **Chunk size**: Variable based on PDF structure

### System Prompt
The AI uses this system prompt template:
```
You are a helpful assistant that explains board game rules. Use the following game rules to answer questions accurately and clearly. If the rules don't contain enough information to answer the question, say so honestly.

Game Rules Context:
{context_chunks}

Instructions:
- Answer based on the provided rules context
- Be concise but thorough
- If rules are unclear or missing, acknowledge this
- Use examples when helpful
- Focus on practical gameplay guidance
```

## Troubleshooting

### Common Issues

**Chat not working:**
- Verify Ollama is running: `curl http://localhost:11434/api/tags`
- Check if Mistral model is downloaded: `ollama list | grep mistral-small3.2:24b`
- Ensure game has uploaded PDF rules

**Poor AI responses:**
- Upload higher quality PDF rules
- Ask more specific questions
- Check if PDF processing completed successfully

**No context found:**
- Verify embeddings were created during PDF upload
- Try broader or different keywords
- Check similarity search results directly

### Debug Information
- Check browser console for API errors
- Monitor backend logs for LLM/embedding issues
- Verify database contains embeddings for the game

### Performance Notes
- First query may be slower (model loading)
- Large PDFs take time to process embeddings
- Context search is optimized with sqlite-vec indexing

## Future Enhancements

Potential improvements for the chat feature:
- **Streaming responses** for real-time chat feel
- **Multi-turn context** preservation across messages
- **Game-specific prompts** for different rule styles
- **Image/diagram support** from PDFs
- **Advanced search filters** (page numbers, sections)
- **Export conversations** for reference
- **Collaborative sessions** between users