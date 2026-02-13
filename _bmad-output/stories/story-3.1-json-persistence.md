# Story 3.1: JSON State Persistence Across Detach/Re-attach

## Epic
Epic 3: Resilience & Persistence

## User Story
As a developer,
I want my summaries, buffer hashes, and sidebar visibility state to persist to disk,
So that I don't lose session context when I detach and re-attach to Zellij.

## Requirements
- FR22: Summaries persist across session detach and re-attach
- FR23: Buffered output persists across session detach and re-attach
- FR24: Sidebar state (visible/hidden) persists across detach and re-attach

## Tasks
- [ ] Create `default-plugins/session-intelligence/src/persistence.rs` module
- [ ] Define PersistedState struct (serde Serialize/Deserialize): version (u32), sidebar_visible (bool), panes (HashMap of pane_id -> PersistedPaneData)
- [ ] Define PersistedPaneData: name, summary_text, status (GREEN/YELLOW/RED string), generated_at, last_scrollback_hash
- [ ] Implement save_state(state: &PluginState, session_name: &str) that serializes to JSON and writes to ~/.local/share/zellij/session-intelligence/{session_name}.json
- [ ] Create the directory if it doesn't exist
- [ ] Set JSON version field to 1
- [ ] Implement load_state(session_name: &str) -> Option<PersistedState> that reads and deserializes JSON
- [ ] If JSON file doesn't exist, return None (plugin starts with default empty state)
- [ ] Call load_state() during plugin initialization to restore previous state
- [ ] Call save_state() after each summary update
- [ ] Call save_state() on BeforeClose event (subscribe to it in main.rs)
- [ ] All filesystem errors logged via eprintln! but never crash the plugin

## Acceptance Criteria
- State saved as JSON to ~/.local/share/zellij/session-intelligence/{session_name}.json
- JSON includes version field (value: 1)
- sidebar_visible state persisted
- Per-pane summary text, status, timestamp, and last_scrollback_hash persisted
- save_state() called on summary update and BeforeClose event
- load_state() restores state on plugin init
- Missing JSON file results in default empty state
- Filesystem errors logged but don't crash plugin
- No data corruption to Zellij's own state or config files

## Technical Context
- Use serde_json for serialization (already in workspace deps)
- FullHdAccess permission enables filesystem access
- session_name can be discovered from Zellij API at runtime
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
