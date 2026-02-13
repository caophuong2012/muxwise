# Story 1.4: Toggle Sidebar Visibility

## Epic
Epic 1: Plugin Foundation & Sidebar Shell

## User Story
As a developer,
I want to toggle the sidebar visibility with a keybinding,
So that I can reclaim screen space when I don't need the sidebar.

## Requirements
- FR17: User can toggle sidebar visibility with a keybinding (show/hide)

## Tasks
- [ ] In main.rs update() method, handle Key event for the toggle keybinding
- [ ] Toggle PluginState.sidebar_visible between true and false on key press
- [ ] In render() method, check sidebar_visible -- render nothing when hidden
- [ ] When sidebar becomes visible again, re-render the full sidebar content

## Acceptance Criteria
- Toggle keybinding hides the sidebar when visible
- Toggle keybinding shows the sidebar when hidden
- sidebar_visible flag in PluginState is updated accordingly
- render() function respects the sidebar_visible flag (renders nothing when hidden)

## Technical Context
- Key events arrive in update() as Event::Key(key)
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- Consider a simple key like 'Tab' or a specific key combo for toggle
