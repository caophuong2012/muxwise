---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-03-core-experience
  - step-04-emotional-response
  - step-05-inspiration
  - step-06-design-system
  - step-07-defining-experience
  - step-08-visual-foundation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - docs/ARCHITECTURE.md
---

# UX Design Specification - Zellij Session Intelligence

**Author:** Tom
**Date:** 2026-02-10

---

## Executive Summary

### Project Vision

Zellij Session Intelligence is a WASM plugin adding an AI-powered session memory sidebar to the Zellij terminal multiplexer. The UX challenge: deliver instant context orientation (<10 seconds) within terminal rendering constraints (text-only, ANSI colors, unicode characters). The sidebar must feel like a native part of Zellij, not a bolt-on. Zero user interaction required -- the UX is passive consumption, not active engagement.

### Target Users

**Primary:** Solo developer, intermediate skill, keyboard-first terminal workflow. Juggles multiple concurrent sessions across projects. Doesn't want to interact with the sidebar -- wants to glance at it. Values speed and minimal friction over configurability or aesthetics.

**Secondary:** Other developers who clone the fork. Same profile but no onboarding relationship with the builder. UX must be self-explanatory.

### Key Design Challenges

1. **Terminal rendering constraints** -- Text characters and ANSI colors only. No icons, variable fonts, or rich graphics. Visual hierarchy must be achieved through color, whitespace, and text formatting (bold, dim).
2. **Narrow sidebar, dense information** -- Must be narrow enough to not steal working pane space, yet display meaningful 2-3 line summaries plus metadata per pane entry.
3. **Glanceability** -- Success = orientation in <10 seconds. Color signals status before text is read. Summary text must communicate in a scan, not a study.
4. **Native integration** -- Must match Zellij's existing visual language: boundary characters, color themes, layout patterns. Should feel like it was always part of Zellij.

### Design Opportunities

1. **Color as primary signal** -- Green/yellow/red status carries enormous visual weight in a text environment. The eye catches color before reading begins.
2. **Always-visible sidebar** -- Zero navigation cost. Context is always present without user action.
3. **Keyboard-native interaction** -- Keybinding toggle and keyboard pane selection fit terminal user expectations perfectly.
4. **Progressive information density** -- Color communicates urgency, summary text communicates context, timestamp communicates freshness -- three layers of information in a single glance.

## Core User Experience

### Defining Experience

The core interaction is the **glance** -- a passive, sub-10-second visual scan of the left sidebar that tells the user what each session is about, where they left off, and what needs attention. This is not an interactive tool; it is an ambient information display. The sidebar is consumed like a dashboard gauge: always visible, always current, never requiring deliberate engagement to deliver value.

### Platform Strategy

- **Platform:** Terminal UI (TUI) within Zellij via WASM plugin
- **Input model:** Keyboard-first. Keybinding for toggle, keyboard/mouse for pane selection
- **Rendering:** ANSI colors, unicode box drawing, text characters only
- **Connectivity:** Requires Claude API access. No offline mode for MVP
- **Integration:** Must coexist with Zellij's existing pane layout, boundaries, tab bar, and status bar

### Effortless Interactions

- **Summaries appear automatically** -- no user action to trigger capture or summarization
- **Sidebar visible by default** -- no navigation required to find it
- **Color communicates before text** -- status is understood at a glance without reading
- **Re-attach triggers refresh** -- returning to a session automatically updates summaries
- **Only two deliberate interactions:** toggle sidebar visibility (keybinding) and navigate to pane (click/keyboard)

### Critical Success Moments

1. **First population** -- Sidebar shows its first useful AI summaries after a few minutes of terminal activity. User sees "it's working" without having done anything.
2. **First return** -- User comes back after being away, glances at sidebar, knows exactly where every session stands. The "aha" moment: "The terminal remembers what I forgot."
3. **First graceful failure** -- API goes down or rate limits hit. Sidebar shows cached summaries with stale indicators instead of crashing or blanking. User trusts the system handles problems.

### Experience Principles

1. **Passive over active** -- The sidebar delivers value through observation, not interaction. If the user has to do something, it's already too much.
2. **Color first, text second** -- Visual hierarchy starts with status color (instant), then summary text (seconds), then timestamp (detail). Design for scan speed.
3. **Native, not bolted-on** -- The sidebar must feel like it was always part of Zellij. Match existing visual language, respect existing layout behavior.
4. **Graceful always** -- Every failure mode (API down, rate limit, stale data) has a visible, non-destructive fallback. The sidebar never makes things worse.

## Desired Emotional Response

### Primary Emotional Goals

1. **Relief** -- "I don't have to remember." The dominant emotion when returning to a session. The sidebar eliminates the cognitive burden of reconstructing context. The user exhales, not scrambles.
2. **Confidence** -- "I know exactly where I am." Each summary provides enough clarity that the user trusts they can resume immediately. No second-guessing, no verification scrolling.
3. **Trust** -- "This is reliable." The sidebar consistently delivers accurate, current information. Timestamps and stale indicators reinforce honesty. The user stops checking its work.

### Emotional Journey Mapping

| Moment | Emotion | Trigger | Design Response |
|---|---|---|---|
| First launch | Curiosity | Empty sidebar appears | Clear empty state: "No session summaries yet." Sets expectation without alarm |
| First summary | Satisfaction | AI summary populates | Color indicator turns green. Content appears. "It works" confirmed silently |
| First return | Relief | Glance at sidebar after being away | Summaries are current, color-coded, scannable. Orientation in seconds |
| Ongoing use | Calm | Sidebar is always there, always current | Consistent layout, predictable behavior, no surprises |
| Long-term | Invisible | Sidebar becomes background infrastructure | User stops noticing it consciously -- it just works |

### Micro-Emotions

- **Confidence over confusion** -- Every pane entry tells you what's happening. No ambiguity, no jargon, no mystery states.
- **Trust over skepticism** -- Timestamps prove freshness. Stale indicators admit when data is old. Honesty builds trust faster than polish.
- **Calm over anxiety** -- Errors are indicated, not alarmed. A stale summary with a dim indicator is calming. A blank panel or crash is anxiety-inducing.

### Design Implications

| Emotional Goal | Design Implication |
|---|---|
| Relief | Sidebar auto-populates without user action. No setup wizard, no "click to summarize" buttons |
| Confidence | Summary language is clear and specific. "Auth refactor paused at middleware decision" not "Session active" |
| Trust | Timestamps on every entry. Stale indicators when data is old. Never show false freshness |
| Calm | Errors shown as dim/subtle indicators, not red alerts. Cached data displayed gracefully, not error messages |
| Invisibility | Visual weight is muted. The sidebar is furniture, not a billboard. It recedes when everything is fine |

### Emotional Design Principles

1. **Remove anxiety, don't add excitement** -- The sidebar's job is to eliminate the stress of "where was I?" Not to impress or delight. Calm utility over clever interaction.
2. **Honest over optimistic** -- Show stale data as stale. Show errors as errors. Never pretend things are fine when they aren't. Users trust honest systems.
3. **Quiet presence** -- The best outcome is the user forgetting the sidebar exists as a separate thing. It becomes part of how Zellij works. Invisible infrastructure.

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

**htop**
- Solves system monitoring with zero configuration -- launch it and information is immediately available
- Color is the primary information channel: green/yellow/red CPU bars communicate load before you read any numbers
- Dense information in a compact space -- every character earns its place
- Real-time updates without user action -- passive consumption model
- Works entirely within terminal constraints and feels native to the environment

**Zellij Status Bar**
- The gold standard for "native feel" -- this is what the sidebar must match
- Compact, always-visible, never intrusive -- occupies minimal space while delivering persistent context
- Uses Zellij's own color theming and text formatting conventions
- Shows mode, tabs, and contextual tips -- layered information density in a single line
- Users forget it's there until they need it -- invisible infrastructure

**GitHub Notifications**
- Color and visual weight signal priority before reading text (blue dot = unread, dimmed = read)
- Categorization by type (PR, issue, review request) enables scanning by intent
- List format with consistent entry structure -- eyes learn the pattern and scan faster over time
- Stale/old notifications are visually deprioritized, not hidden -- honest about age

### Transferable UX Patterns

**Information Density Patterns:**
- htop's color-as-data approach -- status color carries the primary signal, text is secondary detail
- Zellij status bar's "earned space" principle -- every character must justify its presence in a narrow panel

**Passive Consumption Patterns:**
- htop's zero-interaction model -- launch and consume. No buttons, no prompts, no setup
- GitHub notifications' scan-and-go list -- consistent entry format enables rapid visual scanning

**Visual Hierarchy Patterns:**
- htop's color-first hierarchy -- green/yellow/red before numbers
- GitHub's read/unread dimming -- visual weight signals freshness without explicit timestamps
- Zellij status bar's use of bold/dim text to separate primary from secondary information

**Native Integration Patterns:**
- Zellij status bar's seamless visual language -- same border characters, same color palette, same text conventions
- htop's terminal-native rendering -- uses ANSI colors and unicode characters to create rich display within constraints

### Anti-Patterns to Avoid

- **Notification overload (Slack-style)** -- badges, counts, and urgency indicators everywhere. The sidebar should calm, not alarm
- **Interactive dashboards that require clicks** -- htop works because you don't have to click anything. The sidebar must be the same
- **Generic status labels** -- "Session active" tells you nothing. htop shows specific numbers; the sidebar must show specific context
- **Hidden information behind hover/expand** -- everything relevant must be visible at a glance. No tooltips in a terminal
- **Visual noise from borders and decorations** -- keep chrome minimal. Content over container

### Design Inspiration Strategy

**What to Adopt:**
- htop's color-as-primary-signal model -- green/yellow/red status indicators carry the heaviest visual weight
- Zellij status bar's visual language -- boundary characters, color palette, text formatting conventions
- GitHub notifications' consistent list entry structure -- same layout per entry enables learned scanning
- htop's zero-interaction passive display model -- no buttons, no prompts, just information

**What to Adapt:**
- htop's density for a narrower panel -- htop uses the full terminal width; the sidebar must compress to ~25-30 columns
- GitHub's read/unread dimming -- adapt as fresh/stale indicator using dim text and timestamps
- Zellij status bar's single-line format -- expand to multi-line entries while keeping the same visual weight per unit of information

**What to Avoid:**
- Notification-style urgency (red badges, counts, alerts) -- conflicts with the calm/relief emotional goals
- Interactive elements beyond click-to-navigate -- conflicts with passive consumption principle
- Custom visual styling that diverges from Zellij's native look -- conflicts with "native, not bolted-on" principle

## Design System Foundation

### Design System Choice

**Zellij-Native TUI Design System** -- Adopt Zellij's existing visual language as the sidebar's design foundation. The sidebar is a Zellij pane rendered by a WASM plugin; it should look, feel, and behave like any other Zellij UI element.

### Rationale for Selection

- **Native integration is a core principle** -- The sidebar must feel like it was always part of Zellij. Using Zellij's own visual vocabulary is the fastest path to this goal.
- **Solo developer, minimal design overhead** -- No custom design tokens to define or maintain. Follow the host application's existing decisions.
- **Terminal constraints are already solved** -- Zellij has already made good decisions about unicode box drawing, ANSI color usage, and text formatting within terminal limitations.
- **Theming comes free** -- If Zellij supports color themes, the sidebar inherits them automatically.

### Implementation Approach

- **Border characters:** Use Zellij's `boundaries.rs` unicode box-drawing characters for panel edges
- **Color palette:** Map to Zellij's existing ANSI color assignments (foreground, background, accent colors)
- **Text formatting:** Bold for primary text (pane name/project), normal for summary text, dim for timestamps and metadata
- **Status colors:** Green (ANSI green), Yellow (ANSI yellow), Red (ANSI red) -- standard terminal colors that work across themes
- **Spacing:** Single blank line between pane entries. No wasteful padding in a narrow panel.

### Customization Strategy

**Custom elements (additions to Zellij's language):**
- Status color indicator (colored block or dot character preceding each pane entry) -- new visual element not in standard Zellij
- Stale/error indicator (dim text or specific unicode character) -- new semantic meaning
- Summary text wrapping within narrow column constraints -- new layout behavior

**Inherited elements (no customization needed):**
- Panel border rendering
- Background/foreground colors
- Text weight conventions (bold, normal, dim)
- Scroll behavior if entries overflow

## Defining Experience Deep Dive

### The Defining Interaction

**"Glance left, know everything."** The user opens Zellij (or re-attaches), their eyes move left to the sidebar, and within 10 seconds they know: what each session is about, where they left off, and what needs attention. This is a passive read, not an active query. The closest analogue is checking a car's dashboard gauges -- you don't interact with the speedometer, you just see it.

### User Mental Model

**Current mental model (what we're replacing):**
- "I have to reconstruct what I was doing" -- User opens a session, scrolls up, reads terminal history, or re-prompts the AI. The mental model is archaeological: dig through layers to find context.

**Target mental model (what we're creating):**
- "My terminal already knows" -- User expects context to be surfaced, not searched for. The mental model is a briefing: information comes to you, organized and summarized.

**Mental model transition:**
- No education needed. The sidebar is self-explanatory: colored entries with text summaries and timestamps. The user's existing mental model for "list of items with status indicators" (email inbox, notification list, htop process list) transfers directly.

### Success Criteria for Core Experience

| Criterion | Measurement | Threshold |
|---|---|---|
| Orientation speed | Time from eyes-on-sidebar to "I know what's happening" | < 10 seconds for all panes |
| Summary accuracy | Does the summary match what the session is actually doing? | User never has to scroll to verify |
| Zero-action delivery | Did the user have to do anything to get the summary? | No clicks, no commands, no prompts |
| Scan-not-read | Can the user get value from color + layout before reading text? | Color alone communicates status |
| Trust calibration | Does the user trust the sidebar enough to act on it? | User clicks a pane and resumes without checking |

### Novel vs. Established Patterns

**Established patterns (no education needed):**
- List of items with status indicators (email inbox, htop, notification center)
- Color-coded status (green/yellow/red is universal)
- Click-to-navigate from list to target (file browser, IDE sidebar)
- Always-visible sidebar panel (IDE file trees, chat member lists)

**Novel combination (the innovation):**
- AI-generated summaries in a terminal sidebar -- no existing terminal tool does this
- Push-based content population -- summaries appear without user action, unlike IDE panels that require opening/refreshing
- Narrative summaries (story of what happened) vs. status summaries (current state only)

**Education strategy:** None needed. The novel element (AI summaries) appears in a familiar container (sidebar list with status colors). Users understand the container; the content is self-explanatory.

### Experience Mechanics

**1. Initiation -- How does the experience begin?**
- User launches Zellij or re-attaches to a session
- Sidebar is visible by default on the left
- No user action initiates the experience -- it's already there

**2. Information Delivery -- What does the user see?**
- Each pane entry displays (top to bottom):
  - Status color indicator + pane name/project (bold)
  - AI summary: 2-3 lines of narrative text (normal weight)
  - Last activity timestamp (dim)
- Entries are separated by a single blank line
- Active/focused pane is visually highlighted

**3. Feedback -- How does the user know it's working?**
- Color indicators are present (green/yellow/red) -- "it's alive"
- Timestamps update -- "it's current"
- Summaries change when session activity changes -- "it's watching"
- Empty state ("No session summaries yet") confirms the system is running but waiting for content

**4. Navigation -- The only active interaction**
- User clicks or selects a pane entry in the sidebar
- Focus moves to that pane
- This is the only deliberate user action in the entire experience

**5. Degradation -- When things go wrong**
- API unavailable: cached summaries remain with stale indicator (dim timestamp, subtle marker)
- No crash, no blank panel, no error dialog
- User still has the last known good summary -- stale context is better than no context

## Visual Design Foundation

### Color System

**Brand baseline:** Zellij's existing theme colors. The sidebar inherits whatever theme the user has configured.

**Semantic color mapping for sidebar-specific elements:**

| Semantic Role | ANSI Color | Usage |
|---|---|---|
| Status: Active/Healthy | Green | Pane is running, tests passing, no issues |
| Status: Waiting/Attention | Yellow | Waiting for user input, paused, decision pending |
| Status: Error/Problem | Red | Build failed, error state, needs immediate attention |
| Primary text | Default foreground (bold) | Pane name/project identifier |
| Summary text | Default foreground (normal) | AI-generated summary lines |
| Metadata text | Default foreground (dim) | Timestamps, stale indicators |
| Selected/focused entry | Inverse/highlight | Currently highlighted pane entry in sidebar |
| Stale indicator | Dim + yellow | Cached summary that hasn't been refreshed |
| Empty state | Dim | "No session summaries yet" |

**Contrast and readability:** ANSI standard colors are designed for terminal readability across dark and light backgrounds. Green/yellow/red status colors are universally distinguishable. Dim text provides visual hierarchy without losing legibility.

### Typography System

**Terminal typography is fixed:** Monospace font, user-configured. No font choices to make.

**Text weight hierarchy (the only lever available):**

| Weight | ANSI Attribute | Usage | Purpose |
|---|---|---|---|
| Heavy | Bold | Pane name + status indicator | First thing the eye catches |
| Normal | Regular | Summary text (2-3 lines) | Core information, read second |
| Light | Dim | Timestamps, metadata, stale markers | Detail layer, read third or not at all |

**Text formatting rules:**
- No underline (reserved for links in some terminals)
- No blinking (distracting, violates calm principle)
- No italic (inconsistent terminal support)
- Bold, normal, dim only -- three levels of hierarchy is enough

### Spacing & Layout Foundation

**Grid system:** Terminal character cells. Each cell is 1 character wide, 1 line tall. Layout is measured in columns and rows, not pixels.

**Sidebar dimensions:**
- Width: ~30 columns (enough for 2-3 line summaries with word wrapping, narrow enough to preserve working pane space)
- Height: Full terminal height minus status bar and tab bar
- Position: Left edge, separated from content panes by Zellij's standard boundary character

**Pane entry layout (within sidebar):**

```
[color] Pane Name / Project          <- Bold, line 1
  AI summary text wraps across       <- Normal, lines 2-3
  two to three lines maximum.
  2 min ago                           <- Dim, last line
                                      <- Blank line separator
[color] Next Pane Name               <- Next entry begins
```

**Spacing rules:**
- 1 blank line between pane entries
- 2-character left indent for summary and timestamp lines (visual nesting under pane name)
- No right padding beyond natural text end
- Status color indicator: single unicode block character (e.g., `▌`) at left edge of pane name line

### Accessibility Considerations

- **Color is never the only signal** -- Status color is always accompanied by text context in the summary. Color-blind users can still orient via summary text and timestamps.
- **High contrast by default** -- ANSI bold/normal/dim against terminal background provides strong contrast ratios on both dark and light themes.
- **No animation** -- No blinking, scrolling tickers, or moving elements. Static display only.
- **Keyboard accessible** -- Toggle visibility via keybinding. Navigate entries via keyboard. No mouse required.
- **Screen reader compatible** -- Plain text content with logical reading order (pane name -> summary -> timestamp -> next entry).
