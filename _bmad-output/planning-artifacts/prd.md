---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-03-success
  - step-04-journeys
  - step-05-domain-skipped
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
inputDocuments:
  - _bmad-output/planning-artifacts/product-brief-zellij-2026-02-10.md
  - docs/ARCHITECTURE.md
  - docs/ERROR_HANDLING.md
  - docs/MANPAGE.md
  - docs/RELEASE.md
  - docs/TERMINOLOGY.md
  - docs/THIRD_PARTY_INSTALL.md
documentCounts:
  briefs: 1
  research: 0
  brainstorming: 0
  projectDocs: 6
classification:
  projectType: cli_tool
  domain: developer_productivity
  complexity: medium
  projectContext: brownfield
workflowType: 'prd'
---

# Product Requirements Document - Zellij Session Intelligence

**Author:** Tom
**Date:** 2026-02-10

## Executive Summary

Zellij Session Intelligence is a WASM plugin for the Zellij terminal multiplexer that adds an automatic AI-powered session memory sidebar. Developers running multiple concurrent AI coding sessions lose track of what each session was doing when they return after stepping away -- costing 20-30+ minutes daily in re-orientation. This plugin solves the problem by passively capturing terminal output, summarizing it via the Claude API, and surfacing a glanceable left-panel sidebar showing what each session is about, where you left off, and what needs attention. Zero manual input required.

**Target User:** Solo developer juggling multiple concurrent terminal sessions across projects and roles.

**Key Differentiator:** First terminal multiplexer with AI session intelligence. Push-based session memory that requires no user action -- the terminal remembers what you forgot.

**Implementation Strategy:** Zellij WASM plugin to minimize upstream diff and simplify maintenance.

## Success Criteria

### User Success

- Re-orientation time drops from ~5 minutes to under 30 seconds per session switch
- Context recovery via sidebar glance -- no scrolling history or re-prompting AI
- Sidebar is always visible and naturally consulted without conscious effort
- Summaries accurate enough to orient without double-checking
- Zero manual input required

### Business Success

- Tom uses it daily without reverting to tmux
- Fork stays maintainable -- rebasing upstream Zellij changes is not painful
- Another developer can clone, build, and use it from the README alone

### Technical Success

- No noticeable performance degradation in normal Zellij usage
- AI summarization runs on a schedule -- slight latency acceptable
- Session context survives detach/re-attach cycles
- API key is the only required configuration

### Measurable Outcomes

| Metric | Before | After | How to Verify |
|---|---|---|---|
| Re-orientation time | ~5 min/session | <30 seconds | Can you resume from a glance? |
| Context method | Scroll/re-prompt | Sidebar glance | Did you need to scroll or prompt? |
| Manual effort | Notes, re-prompting | Zero | Did you have to do anything? |
| Trust | N/A | Summaries don't mislead | Did the sidebar ever send you wrong? |

## Product Scope

### MVP (Phase 1)

**Approach:** Problem-solving MVP -- the smallest modification that eliminates the 5-minute re-orientation tax.

**Resource Requirements:** Solo developer (Tom), Rust + WASM plugin knowledge, Claude API access.

**Must-Have Capabilities:**
1. Terminal output capture per pane (via plugin API or minimal PTY hook)
2. AI summarization via Claude API (scheduled + on re-attach)
3. Left sidebar panel with per-pane summaries
4. Status color indicators (green/yellow/red)
5. Last activity timestamp per pane
6. Click/select to navigate to pane
7. Toggle sidebar keybinding
8. Summary persistence across detach/re-attach
9. API key configuration in Zellij KDL config
10. Graceful degradation when API is unreachable (cached summary + stale indicator)

**Core User Journeys Supported:**
- Journey 1 (Daily Usage): Full support
- Journey 2 (First Setup): Full support
- Journey 3 (Failure Recovery): Basic support (graceful degradation)
- Journey 4 (Shareability): Full support

### Post-MVP (Not Planned)

These exist only as documented possibilities for future consideration:

- **Phase 2:** Local LLM support for offline/private use; session history timeline
- **Phase 3:** Prompt farm (web portal for multi-session orchestration); team features (shared server, multi-user visibility)

## User Journeys

### Journey 1: Tom Returns After Lunch -- Daily Usage (Happy Path)

Tom has been away for two hours. He had three sessions running: a frontend auth refactor for a client project, a hobby API he's been poking at for weeks, and a blog post draft he started this morning. He opens his terminal.

**Opening Scene:** Tom sits back down at his desk, coffee in hand. He can't remember which session was doing what. Normally he'd start clicking through tabs, scrolling history, or re-prompting Claude with "where was I?"

**Rising Action:** He opens Zellij. The left sidebar is right there. Three entries, each with a 2-3 line AI-generated summary and a color indicator. The frontend session is yellow -- "Auth refactor paused: Claude suggested moving token refresh to middleware. Waiting for your decision on approach." The hobby API is green -- "All 12 endpoint tests passing. Last run 2 hours ago." The blog post is yellow -- "Draft is mid-paragraph in section 3, discussing deployment strategies."

**Climax:** Tom reads the sidebar in about 10 seconds. He clicks the frontend session entry. He's right there, mid-conversation with Claude, and he knows exactly what decision is pending. No scrolling. No re-prompting.

**Resolution:** Tom makes the middleware decision, moves to the blog post to finish the paragraph, and never once lost his train of thought. The 5-minute re-orientation tax is gone.

**Capabilities revealed:** Sidebar display, AI summarization, status color coding, click-to-navigate, auto-refresh.

### Journey 2: Tom Sets Up the Fork -- First-Time Onboarding

Tom has just finished building his modified Zellij. He's ready to switch from tmux.

**Opening Scene:** Tom clones his fork, builds it (`cargo build --release`). He has a Claude API key ready.

**Rising Action:** He adds his API key to the Zellij config file. He launches the modified Zellij. A left sidebar panel appears -- empty at first, showing "No session summaries yet." He opens a few panes, starts a Claude Code session in one, runs some commands in another.

**Climax:** After a few minutes, the sidebar populates. The first AI-generated summary appears. Color indicators turn from grey to green. It's working. No other configuration needed.

**Resolution:** Tom detaches and re-attaches. The summaries persist. He starts using it as his daily driver, tmux forgotten.

**Capabilities revealed:** Simple config (API key only), sidebar initialization, first summary generation, detach/re-attach persistence.

### Journey 3: Tom Hits a Wall -- Edge Case / Failure Recovery

Tom has been using the sidebar for a week. Today his API key hits a rate limit, and he has 12 sessions open.

**Opening Scene:** Tom notices some sidebar entries show stale summaries -- timestamps haven't updated. A few entries show a subtle error indicator.

**Rising Action:** The sidebar still displays the last successful summary for each session. It doesn't crash or blank out. Tom can still orient based on cached summaries.

**Climax:** Tom realizes he's hit a rate limit. He adjusts his summarization interval in config. Summaries start refreshing again. Nothing was lost.

**Resolution:** The system degrades gracefully -- stale data is better than no data. With 12 sessions the sidebar is long but still scannable.

**Capabilities revealed:** Graceful degradation on API failure, summary caching, error indication, configurable summarization interval, handling many sessions.

### Journey 4: A Developer Clones the Fork -- Shareability

A developer named Alex finds Tom's fork on GitHub. They also run multiple AI coding sessions and want the same sidebar.

**Opening Scene:** Alex reads the README. It explains what the fork adds, how to build it, and the single config requirement (API key).

**Rising Action:** Alex clones, builds (`cargo build --release`), adds their API key to config. They launch it and open a few panes.

**Climax:** The sidebar works. Summaries appear. Alex didn't need to ask Tom anything -- the README was enough.

**Resolution:** Alex is up and running. The fork is usable by someone other than its creator.

**Capabilities revealed:** Build-from-source workflow, clear README, no hidden dependencies, standard Zellij config extension.

### Journey Requirements Summary

| Journey | Key Capabilities Revealed |
|---|---|
| Daily Usage | Sidebar display, AI summarization, status colors, click-to-navigate, auto-refresh |
| First Setup | API key config, sidebar initialization, first summary generation, detach/re-attach persistence |
| Failure Recovery | Graceful API degradation, summary caching, error indicators, configurable intervals |
| Shareability | Build-from-source, clear README, no hidden deps, standard config extension |

## Innovation & Novel Patterns

### Detected Innovation Areas

- **First terminal multiplexer with AI session intelligence.** No existing terminal multiplexer (tmux, screen, Zellij, etc.) offers automatic, AI-powered session summarization. New product category within terminal tooling.
- **Push-based session memory.** Existing context-tracking is pull-based (user must remember to act). This inverts the model -- the terminal passively observes and summarizes without user action.
- **Novel combination:** PTY output capture (existing Zellij capability) + AI story summarization (new) + always-visible sidebar UX (new for terminals). Each piece exists independently; combining them is unprecedented.

### Market Context

- No direct competitors in terminal multiplexer space
- Closest analogues: IDE AI context panels (Cursor, Copilot sidebar) -- but those live in IDEs, not terminals
- Low competitive pressure -- personal tool first, open-source share second

### Validation Approach

- **Primary validation:** Tom uses it daily instead of scrolling/re-prompting. If he stops using it, it failed.
- **Summary quality:** "Good enough to orient" is the bar. Iterate on prompts to improve.

## Technical Architecture

### Integration with Existing Zellij

**Key Scoping Decision:** Implement as a **Zellij WASM plugin** rather than core modification. Minimizes upstream diff, simplifies rebasing, leverages existing plugin infrastructure. Escalate to minimal core modification only if plugin API lacks necessary access.

**Integration points:**
- **PTY Bus** (`zellij-server/src/pty.rs`): Capture terminal output per pane
- **Screen** (`zellij-server/src/screen.rs`): Register sidebar as a managed panel
- **Terminal Pane** (`zellij-server/src/panes/terminal_pane.rs`): Access pane buffers for output capture
- **Plugin system** (WASM): Primary implementation target

**New components:**
- Output capture buffer (per pane, configurable size)
- AI summarization engine (HTTP client for Claude API, prompt management, response caching)
- Sidebar renderer (left panel, text display with color indicators)
- Scheduled summarization trigger (timer-based + re-attach event)
- Summary persistence layer (survives detach/re-attach)

### Config Schema

Extend Zellij's KDL configuration with:
- `ai_api_key` -- Claude/Anthropic API key (required)
- `summarization_interval` -- re-summarization frequency (optional, sensible default)
- `buffer_size` -- terminal output buffer per pane (optional, sensible default)

### Command Structure

- One new keybinding: toggle sidebar visibility (show/hide)
- No new CLI subcommands for MVP
- Sidebar click/select navigates to target pane via existing Zellij pane focus

### Implementation Constraints

- **Minimize upstream diff:** Prefer new files over modifying existing ones
- **Non-blocking AI calls:** API calls must be async; never freeze the terminal
- **Cache-first summarization:** Only re-summarize when buffer content has meaningfully changed
- **Graceful degradation:** Show cached summary + stale indicator on API failure; never crash or block

## Risk Mitigation

**Technical Risks:**
- *Plugin API insufficient for PTY access* -- Fallback: add minimal hooks to Zellij core, keep changes small and isolated
- *Async HTTP from WASM plugin* -- Verify plugin API supports async/HTTP; if not, may need sidecar process or minimal core hook
- *Summary quality* -- Iterate on prompts; no Rust changes needed

**Market Risks:**
- None. Personal tool. If Tom uses it, it succeeded.

**Resource Risks:**
- Solo developer. If implementation stalls, unmodified Zellij still works. No downside risk.

## Functional Requirements

### Terminal Output Capture

- **FR1:** System captures terminal output from each active pane automatically without user action
- **FR2:** System buffers captured output per pane up to a configurable buffer size
- **FR3:** System tracks metadata per pane (pane ID, start time, last activity timestamp)

### AI Summarization

- **FR4:** System sends buffered pane output to Claude API for summarization
- **FR5:** System generates a concise session story per pane (what it's about, where you left off, what's pending)
- **FR6:** System triggers summarization on a configurable schedule
- **FR7:** System triggers summarization on session re-attach
- **FR8:** System caches the last successful summary per pane
- **FR9:** System only re-summarizes when buffer content has changed since last summary

### Sidebar Display

- **FR10:** User can view a left sidebar panel showing all active panes
- **FR11:** Sidebar displays an AI-generated summary (2-3 lines) per pane
- **FR12:** Sidebar displays a status color indicator per pane (green = active/healthy, yellow = waiting for input, red = error/needs attention)
- **FR13:** Sidebar displays last activity timestamp per pane
- **FR14:** Sidebar auto-refreshes when new summaries are available
- **FR15:** Sidebar displays an empty state message when no summaries exist yet

### Navigation

- **FR16:** User can select a pane entry in the sidebar to navigate focus to that pane
- **FR17:** User can toggle sidebar visibility with a keybinding (show/hide)

### Configuration

- **FR18:** User can configure their AI API key in Zellij's KDL config file
- **FR19:** User can optionally configure the summarization interval
- **FR20:** User can optionally configure the output buffer size per pane
- **FR21:** System uses sensible defaults for all optional configuration values

### Persistence

- **FR22:** Summaries persist across session detach and re-attach
- **FR23:** Buffered output persists across session detach and re-attach
- **FR24:** Sidebar state (visible/hidden) persists across detach and re-attach

### Error Handling & Degradation

- **FR25:** System displays the last cached summary when the AI API is unreachable
- **FR26:** System displays a stale/error indicator when summaries cannot be refreshed
- **FR27:** System never crashes or blocks normal Zellij operation due to API failure
- **FR28:** System handles rate limiting gracefully without data loss

## Non-Functional Requirements

### Performance

- **NFR1:** Sidebar rendering does not introduce perceptible delay to keystroke responsiveness in any pane
- **NFR2:** AI API calls execute asynchronously and never block terminal input/output
- **NFR3:** Output capture buffering does not noticeably increase memory usage during normal operation
- **NFR4:** Zellij startup time is not noticeably slower with the plugin loaded
- **NFR5:** Sidebar refresh completes without visible flicker or lag

### Security

- **NFR6:** API key is read from config file only; never logged, displayed in UI, or transmitted to any endpoint other than the configured AI API
- **NFR7:** Terminal output buffers sent to the AI API are transmitted over HTTPS

### Integration

- **NFR8:** System communicates with Claude API via standard HTTPS REST calls
- **NFR9:** System handles API response errors (400, 401, 429, 500) without crashing
- **NFR10:** System is compatible with the Anthropic Messages API format
- **NFR11:** API timeout does not block or hang the plugin; requests have a reasonable timeout threshold

### Reliability

- **NFR12:** Plugin failure never crashes the Zellij host process
- **NFR13:** If the plugin encounters an unrecoverable error, Zellij continues operating normally without the sidebar
- **NFR14:** No data corruption to Zellij's own state or config from plugin operation
- **NFR15:** System recovers automatically when API connectivity is restored after an outage
