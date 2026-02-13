# Story 3.3: Graceful API Failure Handling with Stale Indicators

## Epic
Epic 3: Resilience & Persistence

## User Story
As a developer,
I want to see cached summaries with a stale indicator when the API is unreachable or returns errors,
So that I still have useful context even during API failures.

## Requirements
- FR25: System displays the last cached summary when the AI API is unreachable
- FR26: System displays a stale/error indicator when summaries cannot be refreshed
- FR27: System never crashes or blocks normal Zellij operation due to API failure
- FR28: System handles rate limiting gracefully without data loss

## Tasks
- [ ] In WebRequestResult handler, check response status code before parsing
- [ ] On non-200 status (400, 401, 429, 500): preserve last successful summary, set PaneSummary.is_stale = true, log error via eprintln!
- [ ] On HTTP 429 (rate limit): clear pending_request, do NOT retry immediately -- let the next timer cycle handle it
- [ ] On parse failure: keep previous summary, set is_stale = true, log the raw response snippet
- [ ] Update sidebar.rs render_pane_entry() to show stale indicator: use dim text and yellow color for stale summaries
- [ ] When a subsequent API response succeeds, clear is_stale flag (auto-recovery per NFR15)
- [ ] Ensure no panic! anywhere in error handling paths -- use match/if-let with fallback for all fallible operations
- [ ] Test that all API error codes (400, 401, 429, 500) are handled without crashing

## Acceptance Criteria
- On API error, last successful summary preserved (not cleared)
- PaneSummary.is_stale set to true on affected pane
- Stale summaries render with dim text and yellow color indicator
- 429 rate limit backs off to next timer cycle (no retry loop)
- Plugin never panics on any API error
- Normal Zellij operation unaffected during API failures
- Stale flag cleared on next successful response (auto-recovery)

## Technical Context
- WebRequestResult contains status code, headers, body, and context
- Never panic pattern: match/if-let + fallback + eprintln!
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- NFR9: Handle 400, 401, 429, 500 without crashing
- NFR15: Auto-recovery when API connectivity restored
