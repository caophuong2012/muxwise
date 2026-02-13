# Story 3.2: Re-attach Summarization Trigger

## Epic
Epic 3: Resilience & Persistence

## User Story
As a developer,
I want a full summarization cycle to trigger when I re-attach to a session,
So that summaries refresh immediately when I return after being away.

## Requirements
- FR7: System triggers summarization on session re-attach

## Tasks
- [ ] In update(), handle Event::SessionUpdate to detect session re-attachment
- [ ] On re-attach detection, queue ALL known panes for summarization regardless of hash comparison
- [ ] Process the queue using the standard sequential approach (one pane at a time via summarization_queue)

## Acceptance Criteria
- SessionUpdate event detected on re-attach
- All panes queued for summarization regardless of hash
- Summaries update within one full cycle after re-attach

## Technical Context
- SessionUpdate event fires when session state changes (including re-attach)
- Queue all panes by iterating PluginState.panes keys and adding to summarization_queue
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
