# Story 2.5: Click-to-Navigate from Sidebar to Pane

## Epic
Epic 2: AI-Powered Session Summaries

## User Story
As a developer,
I want to click a pane entry in the sidebar to navigate focus to that pane,
So that I can jump directly to the session I need after reading its summary.

## Requirements
- FR16: User can select a pane entry in the sidebar to navigate focus to that pane

## Tasks
- [ ] In update(), handle Event::Mouse for click events in the sidebar area
- [ ] Map click Y coordinate to the corresponding pane entry (accounting for entry height: name line + summary lines + timestamp line + blank separator)
- [ ] When click maps to a valid pane entry, call focus_pane_with_id(pane_id, false, true) to focus that pane
- [ ] Ignore clicks on empty space or between entries

## Acceptance Criteria
- Mouse click on a pane entry triggers focus_pane_with_id with the corresponding pane ID
- Correct pane receives focus in Zellij
- Clicking on empty space or between entries does not trigger navigation
- Clicked entry area maps to a specific pane

## Technical Context
- Mouse events arrive as Event::Mouse(mouse_event) with position coordinates
- focus_pane_with_id(pane_id: PaneId, should_float: bool, should_focus: bool)
- Requires ChangeApplicationState permission
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
