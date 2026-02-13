---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments:
  - docs/ARCHITECTURE.md
  - docs/TERMINOLOGY.md
date: 2026-02-10
author: Tom
---

# Product Brief: zellij

## Executive Summary

**Zellij Session Intelligence** is a modification to the open-source Zellij terminal multiplexer that adds an automatic **session memory sidebar**. Developers running multiple concurrent AI coding sessions lose track of what each session was doing when they step away and return -- costing 20-30+ minutes daily in re-orientation. Manual tracking methods fail because they depend on user discipline. This modification solves the problem by passively capturing terminal output, summarizing it via AI, and surfacing a glanceable left-panel sidebar showing: what each session is about, the story of where you left off, and what needs your attention. Zero manual input. The terminal remembers what you forgot.

---

## Core Vision

### Problem Statement

Developers running multiple simultaneous AI-assisted coding sessions across terminal multiplexer windows lose context when they step away and return. Sessions can be idle for hours or days across a mix of job projects and personal work. There is no built-in mechanism in any terminal multiplexer to tell you the *story* of what a session was doing or where you left off.

### Problem Impact

- 5+ minutes lost per context switch, multiple times daily (20-30+ min/day)
- Manual tracking (markdown notes, BMAD scans) fails because users forget to maintain them
- Re-prompting the AI wastes time and breaks cognitive flow
- Problem scales with the number of concurrent projects and idle time between sessions

### Why Existing Solutions Fall Short

- **tmux/Zellij session naming**: Static labels with no task-level awareness
- **Status bar plugins**: Show generic system info (git branch, time), not semantic session context
- **Manual documentation workflows (e.g., BMAD-scan-to-markdown)**: Effective for disciplined users but requires remembering to run them -- pull-based, goes stale, fails the moment you forget
- **Re-prompting AI**: Defeats the purpose -- costs the time you're trying to save

### Proposed Solution

A **left sidebar panel** built into Zellij that automatically:
1. Captures terminal output via the existing PTY Bus architecture
2. Summarizes session activity through AI (story-level, not just last few lines)
3. Displays a concise, color-coded overview of all sessions -- what it's about, where you left off, what needs attention
4. Requires zero manual input -- push-based, always fresh, always there

### Key Differentiators

- **Fully automatic**: Push-based session memory -- no user action required, never goes stale
- **AI-powered story summarization**: Understands the narrative of your session, not just recent text
- **Built into Zellij**: Leverages existing PTY architecture for fast adoption
- **Glanceable UX**: Color-coded sidebar (green/yellow/red) lets you orient in seconds, not minutes
- **First of its kind**: No terminal multiplexer offers task-level session intelligence

---

## Target Users

### Primary Users

**Persona: "Tom" -- The Multi-Hat Solo Builder**

- **Role:** Entrepreneur / DevOps / Content Writer / Go-to-Market Engineer / Product Owner / Hobby Coder
- **Environment:** Personal server running tmux (migrating to Zellij), multiple concurrent Claude Code sessions
- **Work style:** Juggles job projects (frontend, backend) and hobby projects simultaneously. Switches between sessions throughout the day. Detaches and returns after varying periods -- lunch breaks, overnight, or days later for hobby projects.
- **Session count:** Multiple active sessions at any time across different projects and roles

**Problem Experience:**
- Loses 5+ minutes per context switch, multiple times daily
- Has tried markdown notes for tracking but forgets to maintain them
- When returning to a session, must scroll history or re-prompt Claude Code to remember where he was
- The more hats he wears, the more sessions he runs, the worse the problem gets

**Workarounds tried:**
- Manual markdown tracking notes in repos -- abandoned due to forgetting
- Re-prompting AI for context -- works but wastes time
- No other tools attempted

**Success Vision:**
- Opens Zellij, glances at the left sidebar, instantly knows: "the frontend auth refactor is waiting for my input, the hobby API tests are passing, the blog post draft is mid-paragraph"
- Zero effort to orient. The terminal already knows what he forgot.
- Goes from 5-minute re-orientation to 10-second glance

### Secondary Users

N/A for MVP. Tom is building this for himself. If friends or other developers adopt it later, they would likely share a similar profile: multi-project developers running concurrent AI coding sessions who value automatic context over manual process.

### User Journey

1. **Discovery:** Tom builds this himself as a Zellij fork -- no discovery phase needed for v1
2. **Onboarding:** Install modified Zellij. Sessions automatically start being tracked. Zero configuration needed.
3. **Core Usage:** Open Zellij. Glance at left sidebar. See all sessions with AI-generated summaries and status colors. Click to navigate. Resume work.
4. **"Aha!" Moment:** First time Tom returns after being away overnight, looks at the sidebar, and knows exactly where he left off across all sessions -- without scrolling or prompting.
5. **Long-term:** The sidebar becomes invisible infrastructure -- always there, always current. Tom stops thinking about context tracking entirely. It just works.

---

## Success Metrics

### Personal Success (What matters to Tom)

| Metric | Current State | Target | How to measure |
|---|---|---|---|
| **Re-orientation time** | ~5 min per session | Under 30 seconds | Can you resume work within a glance? |
| **Context recovery method** | Scroll history / re-prompt AI | Glance at sidebar | Did you need to scroll or prompt? If not, it's working |
| **Daily usage** | N/A | Sidebar is always visible, always consulted | Do you look at it naturally, or ignore it? |
| **Trust** | N/A | You trust the summary is accurate | Did the sidebar ever mislead you about session state? |

### Project Success (Hobby project shared on GitHub)

| Metric | Target | Signal |
|---|---|---|
| **Solves Tom's problem** | Tom personally uses it daily without reverting to tmux | If you stop using it, it failed |
| **Maintainability** | Can stay reasonably close to upstream Zellij | If rebasing becomes a nightmare, the approach was wrong |
| **Shareability** | Someone else can clone, build, and use it | README + build instructions are enough for another developer |

### What "Done" Looks Like for MVP

The MVP is successful when:
1. You open Zellij, see the left sidebar with session summaries
2. The summaries are AI-generated and accurate enough to orient you
3. You didn't have to do anything to make it work -- it just ran
4. You stop re-prompting Claude Code with "where was I?"

### What Would Kill It

- Sidebar summaries are inaccurate or useless -- you still have to scroll/re-prompt
- Performance impact -- Zellij becomes noticeably slower
- Maintenance burden -- upstream Zellij changes make the fork too painful to keep

---

## MVP Scope

### Core Features

**1. Terminal Output Capture & Buffering**
- Hook into Zellij's existing PTY Bus (`zellij-server/src/pty.rs`) to capture terminal output per pane
- Buffer recent output per session (configurable buffer size)
- Track session metadata: pane ID, start time, last activity timestamp
- Persist buffer across Zellij re-attach (session survives detach/attach)

**2. AI Summarization Engine**
- Send buffered output to AI API (Claude API / Haiku) for summarization
- Produce a concise session story: what it's about, where you left off, what's pending
- Trigger summarization on: session idle detection, periodic interval, and on re-attach
- Cache summaries to avoid redundant API calls when nothing changed
- Requires user-provided API key in Zellij config

**3. Left Sidebar Panel**
- Dedicated left panel showing all active sessions
- Per session: name/project identifier, AI-generated summary (2-3 lines), status color indicator, last activity timestamp
- Status colors: green (active/healthy), yellow (waiting for input), red (error/needs attention)
- Click/select a session entry to navigate to that pane
- Panel is always visible, auto-refreshes when summaries update
- Toggle-able with a keybinding (show/hide)

### Out of Scope for MVP

- Web portal / remote management UI
- The "prompt farm" multi-session orchestration concept
- Multi-user / team features
- Offline / local LLM support (requires API connectivity for v1)
- Session history timeline (past summaries over time)
- Custom summary templates or formatting options
- Integration with specific AI tools beyond terminal output parsing (no Claude Code plugin API)

### MVP Success Criteria

1. Sidebar displays accurate, AI-generated summaries for all active panes
2. Summaries update automatically without user action
3. Re-orientation time drops from ~5 minutes to under 30 seconds
4. Zero configuration beyond providing an API key
5. No noticeable performance degradation in normal Zellij usage
6. Survives detach/re-attach cycles without losing session context

### Future Vision

- **v2 -- Local LLM support**: Run summarization locally for offline/private use
- **v2 -- Session history**: Timeline of past summaries, see how a project evolved
- **v3 -- Prompt farm**: Web portal to manage and orchestrate multiple AI coding sessions remotely
- **v3 -- Team features**: Shared server, multiple users can see session status
- **Long-term**: The "prompt farm" vision -- a full orchestration layer for AI-assisted development across machines and teams
