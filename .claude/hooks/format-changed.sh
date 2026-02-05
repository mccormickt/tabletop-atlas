#!/bin/bash
# Auto-format files after Edit/Write operations
# Receives JSON input from Claude Code via stdin

set -e

# Read JSON input from stdin
INPUT=$(cat)

# Extract file path from tool_input.file_path using jq
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Exit if no file path found
if [ -z "$FILE_PATH" ]; then
  exit 0
fi

# Exit if file doesn't exist (might have been deleted)
if [ ! -f "$FILE_PATH" ]; then
  exit 0
fi

# Get the file extension
EXT="${FILE_PATH##*.}"

# Get project directory
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"

# Format based on file extension
case "$EXT" in
  rs)
    # Rust files - use cargo fmt
    cargo fmt --manifest-path "$PROJECT_DIR/backend/Cargo.toml" -- "$FILE_PATH" 2>/dev/null || true
    ;;
  ts|js|svelte|css|json|html|md)
    # Frontend files - use prettier if in frontend directory
    if [[ "$FILE_PATH" == *"/frontend/"* ]] || [[ "$FILE_PATH" == "$PROJECT_DIR/frontend/"* ]]; then
      cd "$PROJECT_DIR" && pnpm exec prettier --write "$FILE_PATH" 2>/dev/null || true
    fi
    ;;
esac

exit 0
