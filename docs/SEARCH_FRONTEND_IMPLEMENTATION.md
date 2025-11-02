# Search Frontend Implementation

This document outlines the comprehensive search interface implementation for Tabletop Atlas, including AI-powered semantic search capabilities, reusable components, and global search state management.

## Overview

The search functionality enables users to perform semantic searches across uploaded game rules using AI-powered embeddings. The implementation includes multiple search interfaces, global state management, and keyboard shortcuts for enhanced user experience.

## Architecture

### Core Components

#### 1. API Integration (`frontend/src/api/`)
- **Regenerated API Client**: Updated using `@oxide/openapi-gen-ts` to include search endpoints
- **Search Types**: `SearchResult`, `RulesSearchResponse`, `SearchRulesQueryParams`
- **Binary Upload Support**: Maintained for PDF uploads while adding search functionality

#### 2. Search Components (`frontend/src/lib/components/`)

**RulesSearch.svelte**
- Full-featured search component with inline results
- Configurable for compact or expanded display
- Event dispatching for search results and interactions
- Support for custom result limits and styling

**QuickSearch.svelte**
- Lightweight search with dropdown results
- Auto-complete style interface
- Click-outside handling and keyboard navigation
- Suitable for embedding in headers or sidebars

**SearchModal.svelte**
- Modal overlay for global search across all games
- Game selection sidebar with rules-enabled filtering
- Full search interface with detailed results
- Keyboard shortcuts (Escape to close)

**Header.svelte**
- Unified header component with integrated search
- Adaptive search display based on context
- Mobile-responsive design
- Navigation state management

**HeaderSearch.svelte**
- Header-specific search controls
- Recent searches dropdown
- Global search modal trigger
- Keyboard shortcut indicators

#### 3. State Management (`frontend/src/lib/stores/search.ts`)

**Search Store**
- Global search state using Svelte stores
- Recent searches persistence (localStorage)
- Favorite results management
- Current game context tracking

**Search Utils**
- Modal control functions
- Search history management
- Local storage integration
- Keyboard shortcut initialization

**Features**
- Automatic deduplication of searches
- 20-item search history limit
- 50-item favorites limit
- Cross-session persistence

### Search Interface Types

#### 1. Dedicated Search Page (`/search`)
- Full-screen search interface
- Game selection sidebar
- Advanced search options
- Comprehensive result display
- Search tips and guidance

#### 2. Embedded Game Search
- Context-aware search within game detail pages
- Compact search component
- Inline results display
- Direct integration with game rules

#### 3. Global Search Modal
- Accessible from any page via Cmd/Ctrl+K
- Cross-game search capabilities
- Recent searches quick access
- Mobile-friendly modal design

#### 4. Header Quick Search
- Always-available search trigger
- Recent searches indicator
- Keyboard shortcut display
- Context-sensitive behavior

## Key Features

### 1. Semantic Search
- AI-powered embeddings for understanding query intent
- Natural language question support
- Contextual relevance scoring
- Multiple result ranking algorithms

### 2. User Experience Enhancements
- **Keyboard Shortcuts**: Cmd/Ctrl+K for global search
- **Auto-complete**: Real-time search suggestions
- **Search History**: Persistent recent searches
- **Favorites**: Save important search results
- **Mobile Responsive**: Optimized for all device sizes

### 3. Search Quality Features
- **Similarity Scoring**: Visual relevance indicators
- **Result Metadata**: Context and source information
- **Result Truncation**: Optimized text display
- **Empty States**: Helpful guidance when no results found

### 4. Performance Optimizations
- **Debounced Search**: Prevents excessive API calls
- **Result Caching**: Stores recent search results
- **Lazy Loading**: Efficient component mounting
- **Progressive Enhancement**: Graceful degradation

## API Integration

### Search Endpoint
```typescript
searchRules({
  query: {
    gameId: number,
    query: string,
    limit?: number
  }
})
```

### Response Format
```typescript
interface RulesSearchResponse {
  gameId: number;
  query: string;
  results: SearchResult[];
  totalResults: number;
}

interface SearchResult {
  chunkId: number;
  chunkIndex: number;
  chunkText: string;
  metadata: string;
  similarityScore: number;
}
```

## Implementation Details

### 1. Component Props and Events

**RulesSearch Component**
```typescript
Props:
- gameId: number
- gameName?: string
- placeholder?: string
- maxResults?: number
- showResultsInline?: boolean
- compact?: boolean

Events:
- search: { query, results, totalResults }
- error: string
- resultClick: SearchResult
```

**SearchModal Component**
```typescript
Props:
- isOpen: boolean (bindable)
- initialGameId?: number
- initialQuery?: string

Events:
- close: void
- resultSelect: { result, game }
- gameSelect: Game
```

### 2. State Management Patterns

**Search Store Structure**
```typescript
interface SearchState {
  isModalOpen: boolean;
  recentSearches: SearchHistoryItem[];
  favoriteResults: SearchResult[];
  currentGame: Game | null;
}
```

**Utility Functions**
- `openModal()` / `closeModal()`
- `addToHistory(query, game, resultCount)`
- `addToFavorites(result, game)`
- `setCurrentGame(game)`
- `loadPersistedData()`

### 3. Navigation Integration

**Header Updates**
- All pages now use consistent Header component
- Context-aware search display
- Integrated navigation state management
- Mobile menu preparation

**Route Structure**
```
/search - Dedicated search page
/games/[id] - Game detail with embedded search
All pages - Global search modal available
```

## Usage Examples

### 1. Basic Search Integration
```svelte
<script>
  import { RulesSearch } from '$lib/components';
</script>

<RulesSearch
  gameId={game.id}
  gameName={game.name}
  on:search={(e) => console.log('Search:', e.detail)}
  on:resultClick={(e) => navigateToResult(e.detail)}
/>
```

### 2. Modal Search Trigger
```svelte
<script>
  import { searchUtils } from '$lib/stores/search';
</script>

<button onclick={searchUtils.openModal}>
  Search Rules
</button>
```

### 3. Header Integration
```svelte
<script>
  import { Header } from '$lib/components';
</script>

<Header
  currentGame={game}
  showSearch={game?.rulesPdfPath ? true : false}
/>
```

## Testing and Validation

### Manual Testing Checklist
- [ ] Search modal opens with Cmd/Ctrl+K
- [ ] Game selection filters to rules-enabled games
- [ ] Search queries return relevant results
- [ ] Similarity scores display correctly
- [ ] Recent searches persist across sessions
- [ ] Mobile responsive design works
- [ ] Keyboard navigation functions
- [ ] Error states display appropriately

### API Testing
```bash
# Test search endpoint
curl "http://localhost:8080/api/chat/search-rules?game_id=1&query=setup&limit=3"

# Expected response format
{
  "game_id": 1,
  "query": "setup",
  "results": [...],
  "total_results": 0
}
```

## Future Enhancements

### Planned Features
1. **Advanced Filters**: Filter by similarity score, result type, metadata
2. **Search Analytics**: Track popular queries and improve suggestions
3. **Collaborative Features**: Share search results between users
4. **AI Chat Integration**: Convert search results into conversational responses
5. **Bulk Search**: Search across multiple games simultaneously
6. **Search Export**: Export search results as PDF or markdown

### Performance Improvements
1. **Search Indexing**: Client-side search index for faster queries
2. **Result Prefetching**: Anticipate and cache likely searches
3. **Infinite Scroll**: Load more results on demand
4. **Search Suggestions**: Auto-complete based on game content

### User Experience
1. **Visual Highlights**: Highlight matching terms in results
2. **Search Filters**: Date ranges, game categories, content types
3. **Saved Searches**: Bookmark and organize frequent searches
4. **Search Sharing**: Generate shareable links to search results

## Dependencies

### Frontend Dependencies
- `@oxide/openapi-gen-ts`: API client generation
- Svelte 5: Component framework
- SvelteKit: Application framework
- Tailwind CSS: Styling framework
- shadcn-svelte: UI component library

### Backend Integration
- Dropshot API framework
- SQLite with embeddings support
- Ollama for AI embeddings
- Binary BLOB storage for embeddings

## Troubleshooting

### Common Issues

**Search Returns No Results**
- Verify game has uploaded PDF rules
- Check if embeddings were generated successfully
- Ensure search query is not empty
- Verify API endpoint accessibility

**Modal Not Opening**
- Check keyboard event listeners
- Verify store state management
- Test modal component mounting
- Check for JavaScript errors

**Search History Not Persisting**
- Verify localStorage availability
- Check for JSON serialization errors
- Test cross-session data loading
- Validate search store integration

### Debug Commands
```bash
# Check API health
curl http://localhost:8080/health

# List games with rules
curl http://localhost:8080/api/games | jq '.items[]'

# Test search endpoint
curl "http://localhost:8080/api/chat/search-rules?game_id=ID&query=test&limit=5"
```

## Conclusion

The search frontend implementation provides a comprehensive, user-friendly interface for semantic search across board game rules. The modular component architecture, global state management, and progressive enhancement ensure a robust and scalable search experience that integrates seamlessly with the existing Tabletop Atlas application.
