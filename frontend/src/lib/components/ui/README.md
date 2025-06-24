# UI Components

This directory contains reusable UI components built with Svelte and styled with Tailwind CSS, following the shadcn/ui design system patterns.

## Search Components

### SearchInput

A comprehensive search input component with loading states, clear button, and keyboard shortcuts.

```svelte
<script>
  import { SearchInput } from '$lib/components/ui';
  
  let searchValue = '';
  
  function handleSearch(query) {
    console.log('Searching for:', query);
  }
</script>

<SearchInput
  bind:value={searchValue}
  placeholder="Search games..."
  loading={false}
  keyboardShortcut="⌘K"
  onSearch={handleSearch}
  showClearButton={true}
  size="default"
/>
```

**Props:**
- `value` - The input value (bindable)
- `size` - "sm" | "default" | "lg"
- `loading` - Shows spinner when true
- `showClearButton` - Shows clear button when value exists
- `keyboardShortcut` - Displays keyboard hint (e.g., "⌘K")
- `onSearch` - Called when Enter is pressed
- `onClear` - Called when clear button is clicked

### SearchResult

A consistent component for displaying search results with similarity scores and metadata.

```svelte
<script>
  import { SearchResult } from '$lib/components/ui';
  
  const results = [
    {
      chunkId: "chunk-1",
      chunkText: "Players win by collecting the most victory points...",
      similarityScore: 0.85,
      metadata: "Page 4, Section 2"
    }
  ];
</script>

{#each results as result, index}
  <SearchResult
    chunkId={result.chunkId}
    chunkText={result.chunkText}
    similarityScore={result.similarityScore}
    metadata={result.metadata}
    {index}
    variant="default"
    interactive={true}
    showSimilarity={true}
    showMetadata={true}
    maxTextLength={180}
    on:click={(e) => console.log('Clicked result:', e.detail)}
  />
{/each}
```

**Props:**
- `chunkId` - Unique identifier for the result
- `chunkText` - The main content text
- `similarityScore` - Relevance score (0-1)
- `metadata` - Optional metadata (page numbers, etc.)
- `index` - Display index number
- `variant` - "default" | "compact" | "highlighted"
- `interactive` - Whether the result is clickable
- `showSimilarity` - Show similarity score badge
- `showMetadata` - Show metadata information
- `maxTextLength` - Maximum characters before truncation

### EmptyState

A flexible empty state component for various scenarios.

```svelte
<script>
  import { EmptyState } from '$lib/components/ui';
  
  function handleAction() {
    console.log('Primary action clicked');
  }
  
  function handleSecondaryAction() {
    console.log('Secondary action clicked');
  }
</script>

<EmptyState
  icon="search"
  title="No results found"
  description="Try adjusting your search terms or filters"
  actionText="Clear Filters"
  secondaryActionText="Browse All"
  size="default"
  onAction={handleAction}
  onSecondaryAction={handleSecondaryAction}
/>
```

**Props:**
- `icon` - "search" | "document" | "game" | "upload" | "chat" | "custom"
- `title` - Main heading text
- `description` - Supporting description
- `actionText` - Primary button text
- `secondaryActionText` - Secondary button text
- `size` - "sm" | "default" | "lg"
- `onAction` - Primary button click handler
- `onSecondaryAction` - Secondary button click handler

**Custom Content:**
```svelte
<EmptyState title="Custom Empty State">
  {#snippet icon()}
    <CustomIcon />
  {/snippet}
  
  {#snippet content()}
    <div class="custom-content">
      <p>Custom content here</p>
    </div>
  {/snippet}
  
  {#snippet actions()}
    <CustomButton />
  {/snippet}
</EmptyState>
```

### LoadingSpinner

A consistent loading spinner with optional text.

```svelte
<script>
  import { LoadingSpinner } from '$lib/components/ui';
</script>

<!-- Simple spinner -->
<LoadingSpinner />

<!-- With text -->
<LoadingSpinner text="Loading games..." />

<!-- Different sizes and variants -->
<LoadingSpinner size="sm" variant="muted" />
<LoadingSpinner size="lg" variant="accent" text="Processing..." />
```

**Props:**
- `size` - "xs" | "sm" | "default" | "lg" | "xl"
- `variant` - "default" | "muted" | "accent" | "destructive"
- `text` - Optional loading text

## Usage Examples

### Complete Search Interface

```svelte
<script>
  import { SearchInput, SearchResult, EmptyState, LoadingSpinner } from '$lib/components/ui';
  
  let searchQuery = '';
  let searching = false;
  let results = [];
  
  async function performSearch(query) {
    searching = true;
    try {
      // Your search logic here
      results = await searchAPI(query);
    } finally {
      searching = false;
    }
  }
</script>

<div class="space-y-4">
  <SearchInput
    bind:value={searchQuery}
    placeholder="Search game rules..."
    loading={searching}
    keyboardShortcut="⌘K"
    onSearch={performSearch}
  />
  
  {#if searching}
    <div class="py-8">
      <LoadingSpinner text="Searching..." class="justify-center" />
    </div>
  {:else if results.length > 0}
    <div class="space-y-3">
      {#each results as result, index}
        <SearchResult
          chunkId={result.id}
          chunkText={result.content}
          similarityScore={result.score}
          metadata={result.source}
          {index}
          on:click={() => handleResultClick(result)}
        />
      {/each}
    </div>
  {:else if searchQuery}
    <EmptyState
      icon="search"
      title="No results found"
      description="Try different keywords or check your spelling"
      actionText="Clear Search"
      onAction={() => searchQuery = ''}
    />
  {:else}
    <EmptyState
      icon="search"
      title="Start searching"
      description="Enter a query to search through game rules"
      size="lg"
    />
  {/if}
</div>
```

## Design Principles

- **Consistency**: All components follow the same design patterns and color schemes
- **Accessibility**: Proper ARIA labels, keyboard navigation, and screen reader support
- **Flexibility**: Variants and customization options for different use cases
- **Composability**: Components work well together and can be easily combined
- **Type Safety**: Full TypeScript support with proper type definitions

## Styling

Components use Tailwind CSS classes and CSS custom properties for theming. They automatically adapt to your design system's color scheme and sizing scale.

The components are designed to work seamlessly with the existing shadcn/ui components in this project.