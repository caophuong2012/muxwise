---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-02-12'
inputDocuments:
  - _bmad-output/planning-artifacts/product-brief-zellij-2026-02-10.md
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
  - docs/ARCHITECTURE.md
  - docs/ERROR_HANDLING.md
  - docs/TERMINOLOGY.md
  - docs/MANPAGE.md
  - docs/RELEASE.md
  - docs/THIRD_PARTY_INSTALL.md
workflowType: 'architecture'
project_name: 'zellij'
user_name: 'Tom'
date: '2026-02-12'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

28 functional requirements across 7 categories:

- **Terminal Output Capture (FR1-3):** Per-pane automatic output capture, configurable buffer size, pane metadata tracking (ID, start time, last activity). Architecturally requires tapping into Zellij's PTY Bus output stream from within a WASM plugin -- the core integration point.
- **AI Summarization (FR4-9):** Claude API calls for session story summarization, scheduled + re-attach triggers, caching with change detection. Requires async HTTP from WASM, timer scheduling, and intelligent diffing of buffer content.
- **Sidebar Display (FR10-15):** Left panel with per-pane entries showing AI summary (2-3 lines), status color (green/yellow/red), timestamps, auto-refresh, and empty state. Requires plugin-rendered UI within Zellij's layout system.
- **Navigation (FR16-17):** Click/select sidebar entry to focus target pane, keybinding toggle for sidebar visibility. Requires plugin-to-host pane focus commands.
- **Configuration (FR18-21):** API key, summarization interval, buffer size in KDL config. Sensible defaults for all optional values.
- **Persistence (FR22-24):** Summaries, buffers, and sidebar state survive detach/re-attach. Requires filesystem persistence from plugin.
- **Error Handling (FR25-28):** Cached summary on API failure, stale indicators, never crash host, graceful rate limiting.

**Non-Functional Requirements:**

15 NFRs across 4 categories:

- **Performance (NFR1-5):** No perceptible keystroke delay, async API calls never block I/O, minimal memory overhead from buffering, no startup slowdown, flicker-free sidebar refresh.
- **Security (NFR6-7):** API key never logged/displayed/leaked, HTTPS-only for API communication.
- **Integration (NFR8-11):** Standard HTTPS REST to Claude API, handle all HTTP error codes gracefully, Anthropic Messages API format, reasonable request timeouts.
- **Reliability (NFR12-15):** Plugin failure never crashes host, unrecoverable errors degrade to no-sidebar (not crash), no state corruption, automatic recovery on connectivity restoration.

**Scale & Complexity:**

- Primary domain: Rust WASM plugin / TUI rendering / external API integration
- Complexity level: Medium -- narrow scope (single plugin) but non-trivial WASM sandbox constraints
- Estimated architectural components: 5-6 (output capture, summarization engine, sidebar renderer, persistence layer, configuration, scheduling/timer)

### Technical Constraints & Dependencies

- **WASM sandbox:** Plugin runs in WebAssembly -- must verify that Zellij's plugin API exposes: HTTP client, filesystem access, timer/scheduling, pane output events, pane focus commands
- **Zellij plugin API surface:** The plugin must receive per-pane terminal output events. Existing PTY Bus feeds Screen/TerminalPane. Whether this data is exposed to plugins via events is the critical unknown.
- **Async HTTP from WASM:** Claude API calls must be non-blocking. Zellij plugins may use `http_request` or similar API -- must not block the plugin event loop.
- **Zellij layout system:** Sidebar must register as a managed panel. Zellij plugins can render in panes -- but always-visible left sidebar positioning needs verification.
- **KDL configuration:** Zellij uses KDL format. Plugin configuration is passed through the layout/config system.
- **Rust ecosystem:** anyhow for errors, existing Zellij error patterns (fatal/non_fatal/context). Plugin code uses zellij-tile crate.
- **Upstream maintainability:** Minimize changes to Zellij core. Prefer new files over modifying existing ones. If plugin API is insufficient, any core hooks must be small and isolated.

### Cross-Cutting Concerns Identified

- **Async/non-blocking I/O:** Every external interaction (API calls, filesystem persistence) must be non-blocking. Permeates all components.
- **Error handling & graceful degradation:** Every component must handle failure without crashing the host. Cached/stale data is always preferred over blank/crash states.
- **Persistence lifecycle:** Summaries, buffers, and UI state must serialize/deserialize across detach/re-attach. Affects output capture, summarization cache, and sidebar state.
- **Security:** API key must flow from config to HTTP headers without leaking to logs, UI, or other endpoints. Single concern but touches config, summarization, and error reporting.
- **Performance budget:** Output capture buffering and sidebar rendering must stay within tight performance constraints in a terminal multiplexer where keystroke latency is critical.

## Starter Template Evaluation

### Primary Technology Domain

Rust WASM plugin within the existing Zellij terminal multiplexer codebase (brownfield).

### Starter Options Considered

This is a brownfield project -- the "starter" is Zellij's existing plugin infrastructure. No external starter template applies. The foundation is:

1. **Zellij in-workspace plugin scaffold** (selected): New plugin directory in `default-plugins/`, using `zellij-tile` crate, compiled to `wasm32-wasip1`. Follows patterns established by 13 existing built-in plugins.
2. **External standalone plugin**: Build outside the workspace and load via `file:` URL. Rejected -- loses workspace dependency management and increases build complexity.

### Selected Starter: Zellij In-Workspace Plugin Scaffold

**Rationale for Selection:**
- Follows existing patterns from 13 built-in plugins (strider, status-bar, etc.)
- Workspace-level dependency management via root Cargo.toml
- All required API capabilities verified as available in `zellij-tile`
- No core modifications needed -- pure plugin implementation
- Consistent with PRD's "minimize upstream diff" constraint

**Initialization Command:**

```bash
mkdir -p default-plugins/session-intelligence/.cargo && \
mkdir -p default-plugins/session-intelligence/src
```

### Plugin API Capabilities Verified

**Pane Output Capture:** `PaneRenderReport` event provides `HashMap<PaneId, PaneContents>` with viewport lines, scrollback above/below, and selected text. Also `get_pane_scrollback(pane_id, full)` for on-demand capture. Requires `ReadPaneContents` permission.

**HTTP Requests:** `web_request(url, verb, headers, body, context)` with `WebRequestResult` event returning status, headers, body, and context for request correlation. Non-blocking by design. Requires `WebAccess` permission.

**Timer/Scheduling:** `set_timeout(seconds)` fires `Timer(f64)` event. Chainable for periodic scheduling.

**Pane Navigation:** `focus_pane_with_id(pane_id, should_float, should_focus)` plus `PaneUpdate` events for tracking pane manifest. Requires `ChangeApplicationState` permission.

**Filesystem Persistence:** Full filesystem access via `FullHdAccess` permission.

**Background Workers:** `ZellijWorker` trait with `register_worker!` macro enables background processing separate from the UI event loop.

**UI Rendering:** `Text`, `Table`, `NestedList` components with color/styling support, coordinate-based positioning via `print_text_with_coordinates`.

**Configuration:** KDL config values passed as `BTreeMap<String, String>` to `load()` method.

### Permissions Required

- `ReadPaneContents` -- Read viewport and scrollback from all panes
- `WebAccess` -- Make HTTPS requests to Claude API
- `ChangeApplicationState` -- Focus panes on sidebar click
- `ReadApplicationState` -- Access pane/tab state
- `FullHdAccess` -- Persist summaries to filesystem

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Output capture strategy: `get_pane_scrollback` on-demand with hash-based change detection
- AI model: Claude Haiku via Anthropic Messages API
- Persistence: JSON to Zellij data directory
- Plugin architecture: Main event loop only, no workers

**Important Decisions (Shape Architecture):**
- Sequential API queue (one pane at a time)
- Status color determined by AI in prompt response
- 120-second default summarization interval
- Sidebar rendering via `print_text_with_coordinates`

**Deferred Decisions (Post-MVP):**
- Local LLM support
- Session history timeline
- Advanced prompt tuning
- Custom summary templates

### Data Architecture

| Decision | Choice | Rationale |
|---|---|---|
| Output capture | `get_pane_scrollback(pane_id, true)` on-demand | Simplest approach; pull scrollback when timer fires, no continuous buffering needed |
| Change detection | Hash scrollback text, compare to last summarized hash | Skip API calls for unchanged panes; simple and effective |
| Scrollback limit | Last 2000 lines sent to API (configurable) | Enough context without blowing token limits |
| Persistence format | JSON via serde_json | Human-readable, already in workspace dependencies, easy debugging |
| Persistence location | `~/.local/share/zellij/session-intelligence/` | Follows XDG conventions, alongside Zellij's own data |
| Persistence scope | Summaries, status colors, timestamps, sidebar visibility state | Everything needed to restore sidebar on re-attach |

### API & Communication

| Decision | Choice | Rationale |
|---|---|---|
| AI model | Claude Haiku (claude-haiku-4-5-20251001) | Fastest, cheapest; summary quality sufficient for 2-3 line outputs |
| API format | Anthropic Messages API over HTTPS | Standard, well-documented |
| Request method | `web_request(url, Post, headers, body, context)` | Built-in plugin API, non-blocking |
| Request correlation | `pane_id` in context BTreeMap | Context passed back in WebRequestResult; simple pane-to-response mapping |
| Multi-pane scheduling | Sequential queue, round-robin through changed panes | Naturally rate-limited, simple, avoids 429s |
| Rate limit handling | On 429, back off and retry on next timer cycle | No retry loop; let the next scheduled cycle handle it |
| Error handling | Cache last successful summary, show stale indicator | Graceful degradation per PRD requirements |
| Request timeout | Rely on Zellij's built-in HTTP timeout | No custom timeout logic needed |

### Plugin Architecture

| Decision | Choice | Rationale |
|---|---|---|
| Event loop model | Main ZellijPlugin event loop only, no ZellijWorker | web_request is already async; no benefit from workers at this scale |
| Code organization | 6 modules: main.rs, capture.rs, summarize.rs, sidebar.rs, persistence.rs, state.rs | Clean separation by functional requirement group |
| State management | Central `PluginState` struct in state.rs, passed through event handlers | Single source of truth, simple ownership |
| Event flow | Timer → capture scrollback → check hash → web_request → WebRequestResult → update state → render | Linear, predictable, easy to debug |

### Sidebar Rendering

| Decision | Choice | Rationale |
|---|---|---|
| Rendering method | `print_text_with_coordinates` with Text components | Direct positioning, follows existing plugin patterns |
| Text wrapping | Hard-wrap at sidebar column width, truncate summary to 3 lines max | Predictable layout, no complex wrapping logic |
| Status color source | AI returns GREEN/YELLOW/RED in response, parsed from output | No heuristic code needed; AI determines semantic status |
| Sidebar overflow | Scroll offset controlled by Up/Down keys when sidebar focused | Minimal interaction, handles many panes |
| Color rendering | ANSI green/yellow/red via Text color_range | Standard terminal colors, works across themes |
| Text hierarchy | Bold for pane name, normal for summary, dim for timestamp | Three-level hierarchy per UX spec |

### Scheduling & Triggers

| Decision | Choice | Rationale |
|---|---|---|
| Default interval | 120 seconds (configurable via KDL) | Balance freshness vs API cost |
| Timer implementation | `set_timeout(120.0)`, re-set on each Timer event | Chainable timer pattern, simple periodic scheduling |
| Re-attach detection | Subscribe to `SessionUpdate` event, trigger summarization cycle | Built-in plugin event, no polling needed |
| Change detection | Hash scrollback text string, compare to stored hash per pane | Skip unchanged panes, avoid redundant API calls |

### Prompt Strategy

| Decision | Choice | Rationale |
|---|---|---|
| Prompt location | Const string in summarize.rs | Easy to iterate on wording without structural changes |
| Prompt content | System prompt instructing: summarize session activity, where left off, what needs attention, respond with status color | Covers all PRD summary requirements in one prompt |
| Response parsing | Extract status color keyword + summary text from response | Simple string parsing, no structured output format needed |

### Infrastructure & Deployment

| Decision | Choice | Rationale |
|---|---|---|
| Build | `cargo build --release` in workspace | Standard Rust workspace build, plugin compiles as workspace member |
| Logging | `eprintln!` from plugin to Zellij's log | Zero infrastructure, built-in |
| CI/CD | None for MVP | Personal tool, ship and iterate |
| Monitoring | None for MVP | Use it, observe, fix |

### Decision Impact Analysis

**Implementation Sequence:**
1. Plugin scaffold (Cargo.toml, main.rs with ZellijPlugin skeleton)
2. Configuration loading (API key, interval, buffer size from KDL)
3. Timer scheduling (set_timeout chain)
4. Scrollback capture (get_pane_scrollback + hash comparison)
5. Claude API integration (web_request + response parsing)
6. Sidebar rendering (Text components with color/positioning)
7. Persistence (JSON save/load on state changes and detach/re-attach)
8. Navigation (click/select to focus pane)
9. Toggle keybinding (show/hide sidebar)

**Cross-Component Dependencies:**
- Summarize depends on capture (needs scrollback text)
- Sidebar depends on state (renders from cached summaries)
- Persistence depends on state (serializes the same struct)
- All components depend on configuration (API key, intervals, limits)

## Implementation Patterns & Consistency Rules

### Naming Patterns

| Area | Convention | Example |
|---|---|---|
| Modules | snake_case, singular | `capture.rs`, `summarize.rs` |
| Functions | snake_case, verb-first | `fetch_scrollback()`, `send_summary_request()` |
| Structs | PascalCase | `PluginState`, `PaneSummary`, `SummaryRequest` |
| Fields | snake_case | `pane_id`, `last_hash`, `summary_text` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_INTERVAL_SECS`, `SYSTEM_PROMPT` |
| Enums | PascalCase variants | `PaneStatus::Active`, `PaneStatus::Waiting` |

Standard Rust conventions throughout. No deviations.

### State Structure Pattern

```rust
// state.rs -- single source of truth
struct PluginState {
    config: PluginConfig,                    // from KDL load()
    panes: HashMap<PaneId, PaneData>,        // per-pane tracking
    sidebar_visible: bool,
    scroll_offset: usize,
    summarization_queue: VecDeque<PaneId>,   // sequential queue
    pending_request: Option<PaneId>,         // currently awaiting API response
}

struct PaneData {
    name: String,
    last_scrollback_hash: u64,
    summary: Option<PaneSummary>,
    last_activity: Instant,
}

struct PaneSummary {
    text: String,            // 2-3 line summary
    status: PaneStatus,      // GREEN/YELLOW/RED
    generated_at: Instant,
    is_stale: bool,          // true when API failed on refresh
}
```

All state flows through `PluginState`. No side-channel state.

### Error Handling Pattern

- **Never panic.** Every fallible operation wrapped in match/if-let.
- **API failure:** Set `is_stale = true` on affected pane, keep cached summary, log via `eprintln!`.
- **Parse failure:** Keep previous summary, log the raw response.
- **Filesystem failure:** Continue without persistence, log error. Plugin works without disk.
- **Pattern:** `match result { Ok(v) => use(v), Err(e) => { eprintln!("context: {e}"); fallback } }`

### Event Flow Pattern

```
Timer event
  → for each pane in queue:
    → get_pane_scrollback(pane_id)
    → hash scrollback
    → if hash != last_hash:
      → build API request
      → web_request(url, Post, headers, body, {pane_id: id})
      → set pending_request = Some(pane_id)
      → STOP (one at a time)
    → else: skip, next pane

WebRequestResult event
  → parse response (summary text + status color)
  → update PaneData.summary
  → clear pending_request
  → trigger render
  → process next pane in queue if any

PaneUpdate event
  → sync pane list (add new, remove closed)
  → update pane names

SessionUpdate event
  → trigger full summarization cycle (re-attach)
```

### Persistence JSON Format

```json
{
  "version": 1,
  "sidebar_visible": true,
  "panes": {
    "terminal_1": {
      "name": "frontend-auth",
      "summary": "Auth refactor paused at middleware decision.",
      "status": "YELLOW",
      "generated_at": "2026-02-12T10:30:00Z",
      "last_scrollback_hash": 12345678
    }
  }
}
```

Version field for future-proofing. Flat and obvious structure.

### Enforcement Guidelines

**All AI agents MUST:**
1. Follow Rust naming conventions (snake_case functions/fields, PascalCase types)
2. Never panic -- all errors handled with match/if-let + fallback
3. Route all state changes through `PluginState` -- no side-channel state
4. Use `eprintln!` for all logging -- no other logging mechanism
5. Keep the event flow linear: timer → capture → request → response → render

## Project Structure & Boundaries

### Complete Plugin Directory Structure

```
default-plugins/session-intelligence/
├── .cargo/
│   └── config.toml              # target = "wasm32-wasip1"
├── Cargo.toml                   # plugin manifest
├── LICENSE.md → ../../LICENSE.md # symlink to root license
└── src/
    ├── main.rs                  # ZellijPlugin impl, event routing, register_plugin!
    ├── state.rs                 # PluginState, PaneData, PaneSummary, PluginConfig
    ├── capture.rs               # fetch_scrollback(), hash_scrollback(), change detection
    ├── summarize.rs             # SYSTEM_PROMPT, build_request(), parse_response()
    ├── sidebar.rs               # render_sidebar(), render_pane_entry(), scroll handling
    └── persistence.rs           # save_state(), load_state(), JSON ser/de
```

Runtime persistence data:
```
~/.local/share/zellij/session-intelligence/
└── {session_name}.json          # persisted state per Zellij session
```

### Architectural Boundaries

**Plugin ↔ Zellij Host:**
- Input: Events (`Timer`, `WebRequestResult`, `PaneUpdate`, `SessionUpdate`, `Key`, `Mouse`)
- Output: API calls (`web_request`, `get_pane_scrollback`, `focus_pane_with_id`, `set_timeout`)
- Config: KDL → `BTreeMap<String, String>` in `load()`
- Rendering: `print_text_with_coordinates` in `render(rows, cols)`

**Plugin ↔ Claude API:**
- Outbound: `web_request` with POST to `https://api.anthropic.com/v1/messages`
- Inbound: `WebRequestResult` event with JSON response body
- Auth: `x-api-key` header from config, `anthropic-version` header

**Plugin ↔ Filesystem:**
- Read: `load_state()` on plugin load
- Write: `save_state()` on summary update and before close (`BeforeClose` event)

### Requirements to Structure Mapping

| FR Category | Module | Key Functions |
|---|---|---|
| Terminal Output Capture (FR1-3) | `capture.rs` | `fetch_scrollback()`, `hash_scrollback()` |
| AI Summarization (FR4-9) | `summarize.rs` | `build_request()`, `parse_response()`, `SYSTEM_PROMPT` |
| Sidebar Display (FR10-15) | `sidebar.rs` | `render_sidebar()`, `render_pane_entry()`, `render_empty_state()` |
| Navigation (FR16-17) | `main.rs` | Mouse/key event handling → `focus_pane_with_id()` |
| Configuration (FR18-21) | `state.rs` | `PluginConfig::from_btree()` |
| Persistence (FR22-24) | `persistence.rs` | `save_state()`, `load_state()` |
| Error Handling (FR25-28) | All modules | Match/fallback pattern, `is_stale` flag |

### Data Flow

```
KDL Config → load() → PluginConfig
                         ↓
Timer fires → capture.rs: get_pane_scrollback → hash → changed?
                         ↓ yes
summarize.rs: build_request → web_request(Claude API)
                         ↓
WebRequestResult → summarize.rs: parse_response → PaneSummary
                         ↓
state.rs: update PluginState.panes[id].summary
                         ↓
sidebar.rs: render_sidebar() ← render(rows, cols)
                         ↓
persistence.rs: save_state() → ~/.local/share/zellij/session-intelligence/{session}.json
```

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility:** All decisions are internally consistent. Rust + wasm32-wasip1 + zellij-tile is the standard Zellij plugin stack proven by 13 existing plugins. JSON persistence uses serde_json already in the workspace. `web_request`, `get_pane_scrollback`, and `set_timeout` are all verified in the plugin API. Sequential queue with main event loop avoids concurrency conflicts.

**Pattern Consistency:** Naming follows standard Rust conventions throughout. State flows through single `PluginState` struct. Error handling is uniform (match + fallback, never panic). No contradictions between patterns and decisions.

**Structure Alignment:** Six source files map cleanly to seven FR categories. Boundaries are clear between plugin ↔ host, plugin ↔ API, and plugin ↔ filesystem.

### Requirements Coverage Validation

| Requirement | Covered By | Status |
|---|---|---|
| FR1-3 (Output Capture) | `capture.rs` + `get_pane_scrollback` | Covered |
| FR4-9 (AI Summarization) | `summarize.rs` + `web_request` + Timer | Covered |
| FR10-15 (Sidebar Display) | `sidebar.rs` + `print_text_with_coordinates` | Covered |
| FR16-17 (Navigation) | `main.rs` + `focus_pane_with_id` | Covered |
| FR18-21 (Configuration) | `state.rs` + KDL `load()` | Covered |
| FR22-24 (Persistence) | `persistence.rs` + JSON + `BeforeClose` | Covered |
| FR25-28 (Error Handling) | Match/fallback pattern + `is_stale` flag | Covered |
| NFR1-5 (Performance) | Async `web_request`, on-demand capture, no blocking | Covered |
| NFR6-7 (Security) | API key from config → header only, HTTPS | Covered |
| NFR8-11 (Integration) | Anthropic Messages API, HTTP error handling | Covered |
| NFR12-15 (Reliability) | Never panic, graceful degradation, auto-recovery | Covered |

All 28 functional requirements and 15 non-functional requirements have architectural support. No gaps found.

### Gap Analysis Results

**Critical Gaps:** None.

**Minor Items (address during implementation, not blocking):**
- Exact `anthropic-version` header value -- verify at implementation time
- Sidebar column width -- hardcode 30 initially, make configurable later
- Session name for persistence filename -- discover from Zellij API at runtime

### Architecture Completeness Checklist

**Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**Architectural Decisions**
- [x] Critical decisions documented
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**Implementation Patterns**
- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**Project Structure**
- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**
- Pure plugin implementation, zero core modifications -- minimal upstream diff
- All required API capabilities verified in existing Zellij plugin infrastructure
- Simple, linear event flow -- easy to debug and iterate
- Every decision favors simplicity over cleverness (ship rough, sharpen later)

**Areas for Future Enhancement:**
- ZellijWorker for parallel summarization (if sequential becomes too slow with many panes)
- Local LLM support (post-MVP)
- Configurable prompts (post-MVP)
- Session history timeline (post-MVP)

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all modules
- Respect project structure and boundaries
- Refer to this document for all architectural questions

**Implementation Sequence:**
1. Plugin scaffold (Cargo.toml, main.rs with ZellijPlugin skeleton)
2. Configuration loading (API key, interval, buffer size from KDL)
3. Timer scheduling (set_timeout chain)
4. Scrollback capture (get_pane_scrollback + hash comparison)
5. Claude API integration (web_request + response parsing)
6. Sidebar rendering (Text components with color/positioning)
7. Persistence (JSON save/load on state changes and detach/re-attach)
8. Navigation (click/select to focus pane)
9. Toggle keybinding (show/hide sidebar)
