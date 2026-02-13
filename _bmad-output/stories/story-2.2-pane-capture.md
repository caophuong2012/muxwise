# Story 2.2: Pane Output Capture with Change Detection

## Epic
Epic 2: AI-Powered Session Summaries

## User Story
As a developer,
I want the plugin to capture terminal scrollback per pane and detect when content has changed,
So that only changed panes trigger expensive API calls.

## Requirements
- FR1: System captures terminal output from each active pane automatically without user action
- FR2: System buffers captured output per pane up to a configurable buffer size
- FR3: System tracks metadata per pane (pane ID, start time, last activity timestamp)
- FR9: System only re-summarizes when buffer content has changed since last summary

## Tasks
- [ ] Create `default-plugins/session-intelligence/src/capture.rs` module
- [ ] Implement fetch_scrollback(pane_id) that calls get_pane_scrollback(pane_id, true) and truncates to buffer_size_lines
- [ ] Implement hash_scrollback(text: &str) -> u64 using a fast hash (e.g., DefaultHasher)
- [ ] Add PaneData struct to state.rs with fields: name, last_scrollback_hash (u64), summary (Option<PaneSummary>), last_activity (timestamp)
- [ ] In update(), handle Event::PaneUpdate to sync pane list -- add new panes, remove closed panes, update pane names
- [ ] On Timer event, for each known pane: fetch scrollback, compute hash, compare to last_scrollback_hash
- [ ] Skip panes with unchanged hashes (don't queue for summarization)
- [ ] Add changed panes to PluginState.summarization_queue (VecDeque<PaneId>)
- [ ] Filter out the plugin's own pane from capture (don't summarize the sidebar itself)

## Acceptance Criteria
- get_pane_scrollback(pane_id, true) retrieves scrollback text per pane
- Scrollback is limited to configured buffer_size (default 2000 lines)
- hash_scrollback() computes a hash of the scrollback text
- Hash is compared to PaneData.last_scrollback_hash
- Unchanged panes are skipped (not queued for summarization)
- Changed panes are added to the summarization_queue
- PaneData tracks pane name and last_activity timestamp
- New panes from PaneUpdate events are added to the panes HashMap
- Closed panes are removed from the panes HashMap

## Technical Context
- get_pane_scrollback(pane_id: PaneId, full: bool) returns text content
- Use std::collections::hash_map::DefaultHasher for fast hashing
- PaneUpdate event provides current pane manifest
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
