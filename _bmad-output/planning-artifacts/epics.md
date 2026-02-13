---
stepsCompleted: [1, 2, 3, 4]
status: 'complete'
completedAt: '2026-02-12'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
---

# Zellij Session Intelligence - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Zellij Session Intelligence, decomposing the requirements from the PRD, UX Design, and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: System captures terminal output from each active pane automatically without user action
FR2: System buffers captured output per pane up to a configurable buffer size
FR3: System tracks metadata per pane (pane ID, start time, last activity timestamp)
FR4: System sends buffered pane output to Claude API for summarization
FR5: System generates a concise session story per pane (what it's about, where you left off, what's pending)
FR6: System triggers summarization on a configurable schedule
FR7: System triggers summarization on session re-attach
FR8: System caches the last successful summary per pane
FR9: System only re-summarizes when buffer content has changed since last summary
FR10: User can view a left sidebar panel showing all active panes
FR11: Sidebar displays an AI-generated summary (2-3 lines) per pane
FR12: Sidebar displays a status color indicator per pane (green = active/healthy, yellow = waiting for input, red = error/needs attention)
FR13: Sidebar displays last activity timestamp per pane
FR14: Sidebar auto-refreshes when new summaries are available
FR15: Sidebar displays an empty state message when no summaries exist yet
FR16: User can select a pane entry in the sidebar to navigate focus to that pane
FR17: User can toggle sidebar visibility with a keybinding (show/hide)
FR18: User can configure their AI API key in Zellij's KDL config file
FR19: User can optionally configure the summarization interval
FR20: User can optionally configure the output buffer size per pane
FR21: System uses sensible defaults for all optional configuration values
FR22: Summaries persist across session detach and re-attach
FR23: Buffered output persists across session detach and re-attach
FR24: Sidebar state (visible/hidden) persists across detach and re-attach
FR25: System displays the last cached summary when the AI API is unreachable
FR26: System displays a stale/error indicator when summaries cannot be refreshed
FR27: System never crashes or blocks normal Zellij operation due to API failure
FR28: System handles rate limiting gracefully without data loss

### NonFunctional Requirements

NFR1: Sidebar rendering does not introduce perceptible delay to keystroke responsiveness in any pane
NFR2: AI API calls execute asynchronously and never block terminal input/output
NFR3: Output capture buffering does not noticeably increase memory usage during normal operation
NFR4: Zellij startup time is not noticeably slower with the plugin loaded
NFR5: Sidebar refresh completes without visible flicker or lag
NFR6: API key is read from config file only; never logged, displayed in UI, or transmitted to any endpoint other than the configured AI API
NFR7: Terminal output buffers sent to the AI API are transmitted over HTTPS
NFR8: System communicates with Claude API via standard HTTPS REST calls
NFR9: System handles API response errors (400, 401, 429, 500) without crashing
NFR10: System is compatible with the Anthropic Messages API format
NFR11: API timeout does not block or hang the plugin; requests have a reasonable timeout threshold
NFR12: Plugin failure never crashes the Zellij host process
NFR13: If the plugin encounters an unrecoverable error, Zellij continues operating normally without the sidebar
NFR14: No data corruption to Zellij's own state or config from plugin operation
NFR15: System recovers automatically when API connectivity is restored after an outage

### Additional Requirements

**From Architecture:**
- Plugin scaffold: in-workspace at `default-plugins/session-intelligence/`, compiled to wasm32-wasip1
- Implementation sequence: scaffold → config → timer → capture → API → sidebar → persistence → navigation → toggle
- Permissions required: ReadPaneContents, WebAccess, ChangeApplicationState, ReadApplicationState, FullHdAccess
- Sequential API queue (one pane at a time, round-robin)
- JSON persistence to `~/.local/share/zellij/session-intelligence/{session}.json`
- Status color determined by AI in prompt response (GREEN/YELLOW/RED)
- Claude Haiku model for summarization
- 120-second default summarization interval
- Hash-based change detection on scrollback text
- Never panic -- all errors handled with match/if-let + fallback

**From UX Design:**
- ~30 column sidebar width
- Three text weight levels: bold (pane name), normal (summary), dim (timestamp)
- Status color indicator: colored unicode block character (▌) at left edge
- Single blank line between pane entries
- 2-character left indent for summary and timestamp lines
- Empty state: "No session summaries yet." in dim text
- Stale indicator: dim + yellow for cached summaries
- No animation, no blinking, no underline, no italic
- Screen reader compatible (logical reading order)

### FR Coverage Map

FR1:  Story 2.2 - Pane output capture with change detection
FR2:  Story 2.2 - Pane output capture with change detection
FR3:  Story 2.2 - Pane output capture with change detection
FR4:  Story 2.3 - Claude API integration and summary generation
FR5:  Story 2.3 - Claude API integration and summary generation
FR6:  Story 2.1 - Timer-based summarization scheduling
FR7:  Story 3.2 - Re-attach summarization trigger
FR8:  Story 2.3 - Claude API integration and summary generation
FR9:  Story 2.2 - Pane output capture with change detection
FR10: Story 1.1 - Plugin scaffold with empty sidebar panel
FR11: Story 2.4 - Sidebar summary display with status colors and timestamps
FR12: Story 2.4 - Sidebar summary display with status colors and timestamps
FR13: Story 2.4 - Sidebar summary display with status colors and timestamps
FR14: Story 2.4 - Sidebar summary display with status colors and timestamps
FR15: Story 1.3 - Empty state display
FR16: Story 2.5 - Click-to-navigate from sidebar to pane
FR17: Story 1.4 - Toggle sidebar visibility
FR18: Story 1.2 - Configuration loading from KDL
FR19: Story 1.2 - Configuration loading from KDL
FR20: Story 1.2 - Configuration loading from KDL
FR21: Story 1.2 - Configuration loading from KDL
FR22: Story 3.1 - JSON state persistence across detach/re-attach
FR23: Story 3.1 - JSON state persistence across detach/re-attach
FR24: Story 3.1 - JSON state persistence across detach/re-attach
FR25: Story 3.3 - Graceful API failure handling with stale indicators
FR26: Story 3.3 - Graceful API failure handling with stale indicators
FR27: Story 3.3 - Graceful API failure handling with stale indicators
FR28: Story 3.3 - Graceful API failure handling with stale indicators

## Epic List

### Epic 1: Plugin Foundation & Sidebar Shell
After this epic, the user can load the Session Intelligence plugin, see an empty sidebar panel, toggle its visibility, and configure their API key and preferences. This establishes the complete plugin scaffold and configuration system.
**FRs covered:** FR10, FR15, FR17, FR18, FR19, FR20, FR21

### Epic 2: AI-Powered Session Summaries
After this epic, terminal sessions are automatically captured, summarized by Claude AI on a schedule, and displayed in the sidebar with status colors, timestamps, and click-to-navigate. The core value proposition is fully working.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR8, FR9, FR11, FR12, FR13, FR14, FR16

### Epic 3: Resilience & Persistence
After this epic, session context survives detach/re-attach cycles, the plugin handles API failures gracefully with cached summaries and stale indicators, and it never crashes or blocks Zellij.
**FRs covered:** FR7, FR22, FR23, FR24, FR25, FR26, FR27, FR28

## Epic 1: Plugin Foundation & Sidebar Shell

After this epic, the user can load the Session Intelligence plugin, see an empty sidebar panel, toggle its visibility, and configure their API key and preferences. This establishes the complete plugin scaffold and configuration system.

### Story 1.1: Plugin Scaffold with Empty Sidebar Panel

As a developer,
I want to load the Session Intelligence plugin and see an empty sidebar panel rendered on the left side of Zellij,
So that the plugin infrastructure is in place and visually confirmed working.

**Acceptance Criteria:**

**Given** the session-intelligence plugin directory exists at `default-plugins/session-intelligence/` with Cargo.toml, .cargo/config.toml (target = wasm32-wasip1), and src/main.rs implementing ZellijPlugin trait
**When** the plugin is compiled and loaded in a Zellij layout
**Then** a left-side panel renders at approximately 30 columns wide
**And** the plugin does not crash the Zellij host process
**And** main.rs contains the register_plugin! macro and event subscriptions
**And** state.rs contains the PluginState struct with sidebar_visible (default true) and empty panes HashMap
**And** sidebar.rs contains a render_sidebar() function that renders the panel background

### Story 1.2: Configuration Loading from KDL

As a developer,
I want to configure my API key, summarization interval, and buffer size in Zellij's KDL config file,
So that the plugin reads my preferences on load with sensible defaults for optional values.

**Acceptance Criteria:**

**Given** a Zellij KDL config with `ai_api_key`, `summarization_interval`, and `buffer_size` values in the plugin configuration block
**When** the plugin's `load()` method is called with the config BTreeMap
**Then** the API key is read and stored in PluginConfig
**And** summarization_interval defaults to 120 seconds if not provided
**And** buffer_size defaults to 2000 lines if not provided
**And** provided values override the defaults
**And** a missing API key logs a warning via eprintln! but does not crash the plugin
**And** PluginConfig is stored in PluginState.config

### Story 1.3: Empty State Display

As a developer,
I want to see "No session summaries yet." in dim text when no summaries exist,
So that I know the plugin is running and waiting for content.

**Acceptance Criteria:**

**Given** the plugin is loaded and no pane summaries have been generated
**When** the sidebar renders
**Then** "No session summaries yet." displays in dim text
**And** the message is vertically positioned in the upper area of the sidebar
**And** the empty state message disappears when the first pane summary arrives

### Story 1.4: Toggle Sidebar Visibility

As a developer,
I want to toggle the sidebar visibility with a keybinding,
So that I can reclaim screen space when I don't need the sidebar.

**Acceptance Criteria:**

**Given** the sidebar is currently visible
**When** the toggle keybinding is pressed
**Then** the sidebar hides and working panes expand to fill the space
**And** pressing the toggle keybinding again restores the sidebar
**And** the sidebar_visible flag in PluginState is updated accordingly
**And** the render() function respects the sidebar_visible flag (renders nothing when hidden)

## Epic 2: AI-Powered Session Summaries

After this epic, terminal sessions are automatically captured, summarized by Claude AI on a schedule, and displayed in the sidebar with status colors, timestamps, and click-to-navigate. The core value proposition is fully working.

### Story 2.1: Timer-Based Summarization Scheduling

As a developer,
I want the plugin to fire a summarization cycle on a configurable interval,
So that summaries stay current without any manual action.

**Acceptance Criteria:**

**Given** the plugin is loaded with a configured summarization interval (default 120 seconds)
**When** the plugin starts
**Then** set_timeout is called with the configured interval
**And** on each Timer event, a summarization cycle begins
**And** set_timeout is re-called at the end of each Timer handler to maintain the periodic schedule
**And** the interval value comes from PluginConfig

### Story 2.2: Pane Output Capture with Change Detection

As a developer,
I want the plugin to capture terminal scrollback per pane and detect when content has changed,
So that only changed panes trigger expensive API calls.

**Acceptance Criteria:**

**Given** a Timer event fires and there are active panes tracked via PaneUpdate events
**When** capture.rs fetch_scrollback() is called for each pane
**Then** get_pane_scrollback(pane_id, true) retrieves the scrollback text
**And** scrollback is limited to the configured buffer_size (default 2000 lines)
**And** hash_scrollback() computes a hash of the scrollback text
**And** the hash is compared to PaneData.last_scrollback_hash
**And** panes with unchanged hashes are skipped (not queued for summarization)
**And** panes with changed hashes are added to the summarization_queue
**And** PaneData tracks pane name and last_activity timestamp per pane
**And** new panes from PaneUpdate events are added to the panes HashMap
**And** closed panes are removed from the panes HashMap

### Story 2.3: Claude API Integration and Summary Generation

As a developer,
I want the plugin to send changed pane output to Claude API and receive a concise session summary with status color,
So that each pane gets an AI-generated context story.

**Acceptance Criteria:**

**Given** there are panes in the summarization_queue with changed content
**When** the next pane is dequeued for summarization
**Then** build_request() in summarize.rs constructs an Anthropic Messages API request body
**And** the request is sent via web_request(url, Post, headers, body, context) where context contains the pane_id
**And** headers include x-api-key (from config) and anthropic-version
**And** the model is set to claude-haiku-4-5-20251001
**And** the system prompt instructs the AI to: summarize what the session is about, where the user left off, what's pending, and return STATUS: GREEN, YELLOW, or RED
**And** pending_request is set to Some(pane_id) to block further requests until response arrives
**And** on WebRequestResult, parse_response() extracts the summary text and status color
**And** PaneData.summary is updated with a new PaneSummary (text, status, generated_at, is_stale=false)
**And** PaneData.last_scrollback_hash is updated to the current hash
**And** pending_request is cleared and the next queued pane is processed
**And** the API key is never logged via eprintln! or displayed in the sidebar

### Story 2.4: Sidebar Summary Display with Status Colors and Timestamps

As a developer,
I want to see each pane's AI summary, status color indicator, and last activity timestamp in the sidebar,
So that I can orient myself at a glance when returning to my sessions.

**Acceptance Criteria:**

**Given** one or more panes have AI-generated summaries in PluginState
**When** the sidebar renders
**Then** each pane entry displays a colored unicode block character (▌) at the left edge in the correct status color (green/yellow/red)
**And** the pane name renders in bold text on the same line as the status indicator
**And** the AI summary text (2-3 lines) renders in normal weight with 2-character left indent
**And** the last activity timestamp renders in dim text with 2-character left indent
**And** a single blank line separates each pane entry
**And** entries render in consistent order (by pane ID)
**And** the sidebar auto-refreshes when PluginState is updated with new summaries
**And** the empty state message from Story 1.3 is replaced by pane entries

### Story 2.5: Click-to-Navigate from Sidebar to Pane

As a developer,
I want to click a pane entry in the sidebar to navigate focus to that pane,
So that I can jump directly to the session I need after reading its summary.

**Acceptance Criteria:**

**Given** the sidebar is visible with one or more pane entries displayed
**When** the user clicks on a pane entry in the sidebar
**Then** focus_pane_with_id is called with the corresponding pane ID
**And** the target pane receives focus in Zellij
**And** clicking on empty space or between entries does not trigger navigation
**And** the clicked pane entry is visually identifiable (the entry area maps to a specific pane)

## Epic 3: Resilience & Persistence

After this epic, session context survives detach/re-attach cycles, the plugin handles API failures gracefully with cached summaries and stale indicators, and it never crashes or blocks Zellij.

### Story 3.1: JSON State Persistence Across Detach/Re-attach

As a developer,
I want my summaries, buffer hashes, and sidebar visibility state to persist to disk,
So that I don't lose session context when I detach and re-attach to Zellij.

**Acceptance Criteria:**

**Given** the plugin has generated summaries for one or more panes
**When** save_state() in persistence.rs is called
**Then** the state is serialized as JSON to `~/.local/share/zellij/session-intelligence/{session_name}.json`
**And** the JSON includes a version field (value: 1) for future compatibility
**And** sidebar_visible state is persisted
**And** per-pane summary text, status color, generated_at timestamp, and last_scrollback_hash are persisted
**And** save_state() is called on each summary update and on BeforeClose event
**And** on plugin load, load_state() reads the persisted JSON and restores PluginState
**And** if the JSON file does not exist, the plugin starts with default empty state
**And** filesystem errors (read or write) are logged via eprintln! but never crash the plugin
**And** no data corruption occurs to Zellij's own state or config files

### Story 3.2: Re-attach Summarization Trigger

As a developer,
I want a full summarization cycle to trigger when I re-attach to a session,
So that summaries refresh immediately when I return after being away.

**Acceptance Criteria:**

**Given** the plugin is loaded and a session re-attach occurs
**When** a SessionUpdate event is received indicating re-attachment
**Then** all known panes are queued for summarization regardless of hash comparison
**And** the summarization cycle processes the queue using the standard sequential approach
**And** summaries update within one full cycle after re-attach

### Story 3.3: Graceful API Failure Handling with Stale Indicators

As a developer,
I want to see cached summaries with a stale indicator when the API is unreachable or returns errors,
So that I still have useful context even during API failures.

**Acceptance Criteria:**

**Given** a pane has a previously cached summary and the Claude API returns an error
**When** WebRequestResult returns a non-200 status (400, 401, 429, 500) or times out
**Then** the last successful summary is preserved (not cleared or overwritten)
**And** PaneSummary.is_stale is set to true on the affected pane
**And** stale summaries render with dim text and yellow color indicator in the sidebar
**And** on HTTP 429 (rate limit), the plugin backs off to the next timer cycle without a retry loop
**And** the plugin never panics on any API error -- all failures handled with match/if-let + fallback
**And** normal Zellij terminal operation (keystroke input, pane rendering) is completely unaffected during API failures
**And** when the API returns a successful response after a failure period, is_stale is cleared and the summary updates normally (auto-recovery)
