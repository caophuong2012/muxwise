# Story 1.1: Plugin Scaffold with Empty Sidebar Panel

## Epic
Epic 1: Plugin Foundation & Sidebar Shell

## User Story
As a developer,
I want to load the Session Intelligence plugin and see an empty sidebar panel rendered on the left side of Zellij,
So that the plugin infrastructure is in place and visually confirmed working.

## Requirements
- FR10: User can view a left sidebar panel showing all active panes

## Tasks
- [ ] Create `default-plugins/session-intelligence/Cargo.toml` with zellij-tile dependency and wasm32-wasip1 target
- [ ] Create `default-plugins/session-intelligence/.cargo/config.toml` with `target = "wasm32-wasip1"`
- [ ] Create `default-plugins/session-intelligence/src/main.rs` implementing ZellijPlugin trait with register_plugin! macro
- [ ] Subscribe to required events: Timer, Key, Mouse, PaneUpdate, SessionUpdate, WebRequestResult
- [ ] Create `default-plugins/session-intelligence/src/state.rs` with PluginState struct (sidebar_visible: bool default true, panes: HashMap)
- [ ] Create `default-plugins/session-intelligence/src/sidebar.rs` with render_sidebar() function that renders a left panel at ~30 columns
- [ ] Request permissions: ReadPaneContents, WebAccess, ChangeApplicationState, ReadApplicationState, FullHdAccess
- [ ] Verify plugin compiles to wasm32-wasip1 with `cargo build --target wasm32-wasip1`
- [ ] Verify plugin loads in Zellij without crashing the host process

## Acceptance Criteria
- Plugin directory exists at `default-plugins/session-intelligence/`
- Plugin compiles to wasm32-wasip1
- Plugin loads in a Zellij layout and renders a left-side panel at ~30 columns
- Plugin does not crash the Zellij host process
- main.rs contains register_plugin! macro and event subscriptions
- state.rs contains PluginState struct with sidebar_visible (default true) and empty panes HashMap
- sidebar.rs contains render_sidebar() function that renders the panel background

## Technical Context
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- Follow existing plugin patterns from default-plugins/ (e.g., strider, status-bar)
- Use zellij-tile crate from workspace
- Plugin renders via print_text_with_coordinates in render(rows, cols)
