# Story 2.4: Sidebar Summary Display with Status Colors and Timestamps

## Epic
Epic 2: AI-Powered Session Summaries

## User Story
As a developer,
I want to see each pane's AI summary, status color indicator, and last activity timestamp in the sidebar,
So that I can orient myself at a glance when returning to my sessions.

## Requirements
- FR11: Sidebar displays an AI-generated summary (2-3 lines) per pane
- FR12: Sidebar displays a status color indicator per pane (green/yellow/red)
- FR13: Sidebar displays last activity timestamp per pane
- FR14: Sidebar auto-refreshes when new summaries are available

## Tasks
- [ ] Implement render_pane_entry() in sidebar.rs that renders a single pane entry with: status color block, pane name, summary, timestamp
- [ ] Render status color unicode block character (▌) at left edge in correct ANSI color (green for Active, yellow for Waiting, red for Error)
- [ ] Render pane name in bold text on the same line as the status indicator
- [ ] Render AI summary text (2-3 lines) in normal weight with 2-character left indent
- [ ] Hard-wrap summary text at sidebar column width
- [ ] Render last activity timestamp in dim text with 2-character left indent
- [ ] Add single blank line between pane entries
- [ ] Update render_sidebar() to iterate PluginState.panes and call render_pane_entry() for each pane with a summary
- [ ] Render entries in consistent order (sorted by pane ID)
- [ ] Replace empty state message when at least one pane has a summary
- [ ] Sidebar re-renders automatically when render() is called after state updates

## Acceptance Criteria
- Status color unicode block (▌) renders at left edge in correct color (green/yellow/red)
- Pane name renders in bold text
- Summary text renders in normal weight with 2-char indent
- Timestamp renders in dim text with 2-char indent
- Single blank line between entries
- Entries render in consistent order (by pane ID)
- Sidebar auto-refreshes when new summaries arrive
- Empty state message replaced by pane entries

## Technical Context
- Use Text component with color_range for status colors
- print_text_with_coordinates(text, x, y) for positioning
- UX spec: ~30 column width, bold/normal/dim text hierarchy
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- UX design: _bmad-output/planning-artifacts/ux-design-specification.md
