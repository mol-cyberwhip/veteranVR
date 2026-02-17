# YouTube Trailer Preview Feature

## Context
The windows-sideloader has a feature that shows YouTube trailers when selecting a game. We want to replicate this in the Tauri desktop app. When a user clicks on a game card in the library, a detail modal should appear showing game info and an embedded YouTube trailer.

## Approach

### 1. Add Rust backend command: `search_youtube_trailer`
**File:** `desktop/src-tauri/src/ipc/commands.rs`

- New IPC command that takes a game name, searches YouTube (`https://www.youtube.com/results?search_query={name} VR trailer`), parses the HTML to extract the best video ID using the same scoring logic as windows-sideloader
- Uses reqwest (already in Cargo.toml) with a 5s timeout
- Caches results in-memory (HashMap in AppState) to avoid repeated lookups
- Returns `Option<String>` (video ID or null)

### 2. Regenerate TypeScript bindings
Run `cargo test --lib generate_bindings` and copy to frontend.

### 3. Add API method in frontend
**File:** `desktop/frontend/src/services/api.ts`

- Add `searchYoutubeTrailer(gameName: string): Promise<string | null>`

### 4. Create GameDetailModal component
**File:** `desktop/frontend/src/components/Library/GameDetailModal.tsx`

- Full-screen modal overlay (dark backdrop) that appears when clicking a game card
- Layout: Left side has game thumbnail + info, right side has YouTube embed (16:9 iframe)
- Shows: game name, package name, size, version, download count, release notes
- YouTube iframe using `https://www.youtube.com/embed/{videoId}` with autoplay=1, mute=1
- Loading state while fetching trailer
- "No trailer found" fallback
- Close button + click-outside-to-close + Escape key
- Action buttons: Download & Install, Download Only

### 5. Update GameCard click behavior
**File:** `desktop/frontend/src/components/Library/GameCard.tsx`

- Change `handleCardClick` to open the detail modal instead of toggling inline expansion
- Remove the inline expanded state and expanded content (the modal replaces it)

### 6. Add modal CSS styles
**File:** `desktop/frontend/src/styles/main.css`

- Modal overlay, content panel, YouTube embed container, close button, responsive layout

## Files to Modify
1. `desktop/src-tauri/src/ipc/commands.rs` - Add `search_youtube_trailer` command + YouTube search/parse logic
2. `desktop/src-tauri/src/lib.rs` - Register new command in handler
3. `desktop/frontend/src/services/api.ts` - Add API method
4. `desktop/frontend/src/components/Library/GameDetailModal.tsx` - **NEW** modal component
5. `desktop/frontend/src/components/Library/GameCard.tsx` - Open modal on click
6. `desktop/frontend/src/components/views/Library.tsx` - Manage modal state, render modal
7. `desktop/frontend/src/styles/main.css` - Modal styles
8. Regenerate bindings after Rust changes

## Verification
1. `cd desktop/src-tauri && cargo test --lib generate_bindings` - Regenerate bindings
2. `cd desktop/frontend && npx tsc --noEmit` - Type check
3. `cd desktop && cargo tauri dev` - Run app, click a game card, verify modal opens with trailer
