# Story 1.3: Empty State Display

## Epic
Epic 1: Plugin Foundation & Sidebar Shell

## User Story
As a developer,
I want to see "No session summaries yet." in dim text when no summaries exist,
So that I know the plugin is running and waiting for content.

## Requirements
- FR15: Sidebar displays an empty state message when no summaries exist yet

## Tasks
- [ ] Add render_empty_state() function to sidebar.rs that renders "No session summaries yet." in dim text
- [ ] Position the empty state message in the upper area of the sidebar
- [ ] In render_sidebar(), check if panes HashMap has any summaries -- if none, call render_empty_state()
- [ ] When the first pane summary arrives, render_sidebar() switches to showing pane entries instead of empty state

## Acceptance Criteria
- "No session summaries yet." displays in dim text when no summaries exist
- Message is vertically positioned in the upper area of the sidebar
- Empty state message disappears when the first pane summary arrives

## Technical Context
- Use Text component with dim styling (ANSI dim attribute)
- UX spec: empty state in dim text, no animation, no blinking
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
