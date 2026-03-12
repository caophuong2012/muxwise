# Muxwise

**A terminal multiplexer that remembers what your sessions are doing.**

Built on [Zellij](https://github.com/zellij-org/zellij) (v0.44.0), Muxwise adds an AI-powered **Session Intelligence sidebar** that automatically summarizes your terminal panes so you never lose context — even after stepping away for hours.

## Features

- **AI-generated summaries** — Uses Claude Haiku or GPT-4o-mini to summarize each pane's scrollback into 2-3 actionable lines
- **Status colors** — GREEN (active/healthy), YELLOW (idle/waiting), RED (errors need attention)
- **Click-to-navigate** — Click any pane entry in the sidebar to jump to it
- **Persistent state** — Summaries survive detach/re-attach across sessions
- **Token usage tracking** — See cumulative API cost in the sidebar footer
- **Toggle visibility** — Press `Tab` to show/hide, `s` to manually trigger a scan
- **All of Zellij** — Layouts, floating panes, WASM plugins, web client, and everything else Zellij offers

## Installation

### Option 1: Download prebuilt binary

Download from [Releases](https://github.com/caophuong2012/muxwise/releases) for your platform, extract, and add to your PATH:

```bash
# macOS (Apple Silicon)
tar xzf muxwise-aarch64-apple-darwin.tar.gz
sudo mv zellij /usr/local/bin/muxwise

# macOS (Intel)
tar xzf muxwise-x86_64-apple-darwin.tar.gz
sudo mv zellij /usr/local/bin/muxwise

# Linux (x86_64)
tar xzf muxwise-x86_64-unknown-linux-musl.tar.gz
sudo mv zellij /usr/local/bin/muxwise

# Linux (aarch64)
tar xzf muxwise-aarch64-unknown-linux-musl.tar.gz
sudo mv zellij /usr/local/bin/muxwise
```

### Option 2: Build from source

#### Prerequisites

**Install Rust** (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**Install system dependencies:**

```bash
# Ubuntu / Debian
sudo apt update && sudo apt install -y build-essential protobuf-compiler pkg-config libssl-dev

# Fedora / RHEL
sudo dnf install -y gcc protobuf-compiler pkg-config openssl-devel

# Arch Linux
sudo pacman -S base-devel protobuf pkg-config openssl

# macOS (via Homebrew)
brew install protobuf
```

**Add the WASM build target:**

```bash
rustup target add wasm32-wasip1
```

#### Build

```bash
git clone https://github.com/caophuong2012/muxwise.git
cd muxwise
cargo xtask build --release
```

The binary will be at `target/release/zellij`. Copy it to your PATH:

```bash
sudo cp target/release/zellij /usr/local/bin/muxwise
```

### Run

```bash
muxwise
```

The Session Intelligence sidebar is included in the default layout — just launch and it's there.

## Configuration

Add your API key to `~/.config/zellij/config.kdl`. Muxwise supports two providers:

### Anthropic (default) — uses Claude Haiku

```kdl
plugins {
    session-intelligence location="zellij:session-intelligence" {
        ai_api_key "sk-ant-..."          // your Anthropic API key
        ai_provider "anthropic"          // default, can be omitted
        summarization_interval "60"      // seconds between scans (default: 60)
        buffer_size "2000"               // max scrollback lines to capture (default: 2000)
        cooldown "30"                    // min seconds between re-summarizing same pane (default: 30)
    }
}
```

Get an API key at [console.anthropic.com](https://console.anthropic.com/)

### OpenAI — uses GPT-4o-mini

```kdl
plugins {
    session-intelligence location="zellij:session-intelligence" {
        ai_api_key "sk-..."              // your OpenAI API key
        ai_provider "openai"
    }
}
```

Get an API key at [platform.openai.com](https://platform.openai.com/)

### Configuration options

| Option | Default | Description |
|--------|---------|-------------|
| `ai_api_key` | *(required)* | API key for your chosen provider |
| `ai_provider` | `"anthropic"` | `"anthropic"` or `"openai"` |
| `summarization_interval` | `"60"` | Seconds between automatic scans |
| `buffer_size` | `"2000"` | Max scrollback lines to capture per pane |
| `cooldown` | `"30"` | Min seconds before re-summarizing the same pane |

For general Zellij configuration, see the [Zellij Configuration Documentation](https://zellij.dev/documentation/configuration.html).

## Security

Muxwise sends terminal scrollback to an external AI API for summarization. To protect sensitive data, a **built-in sanitizer** automatically redacts common secret patterns before anything leaves your machine:

- **Environment variables** — `export API_KEY=...`, `DATABASE_URL=...`, and 30+ known secret key names
- **API tokens** — OpenAI (`sk-`), Anthropic (`sk-ant-`), GitHub (`ghp_`, `gho_`), Slack (`xoxb-`, `xoxp-`), AWS (`AKIA`), npm, PyPI
- **JWT tokens** — `eyJ...` base64-encoded tokens
- **Private keys** — PEM-encoded `-----BEGIN ... PRIVATE KEY-----` blocks
- **Connection strings** — URLs containing credentials

This is **best-effort, not a guarantee**. If you work with highly sensitive systems, consider:
- Not configuring an API key (the sidebar still shows pane names, just no summaries)
- Using a self-hosted LLM instead of a cloud API
- Avoiding `cat .env` or printing secrets in monitored panes

## How it works

1. Timer fires every N seconds (default 60s)
2. Plugin fetches scrollback from each terminal pane
3. Hash-based change detection skips unchanged panes
4. Sends changed panes to AI for summarization (one at a time, to avoid rate limits)
5. Sidebar updates with color-coded entries
6. State persists to `~/.local/share/zellij/session-intelligence/`

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Toggle sidebar visibility |
| `s` | Manually trigger a summarization scan |
| Click | Navigate to the clicked pane |

## Roadmap

### What works today
- [x] AI-generated summaries of terminal pane scrollback
- [x] Status colors (GREEN/YELLOW/RED) based on session state
- [x] Sidebar with click-to-navigate between panes
- [x] State persistence across detach/re-attach
- [x] Token usage tracking
- [x] Support for Anthropic (Claude Haiku) and OpenAI (GPT-4o-mini)
- [x] Built-in as a default plugin — no setup beyond adding your API key

### What we're exploring next
- [x] **Scrollback snapshots** — saves captured scrollback to disk (sanitized) before it gets destroyed by terminal compaction or `/clear`, so context survives reboots
- [ ] **Summary history** — see how a session evolved over time, not just the latest state. Useful for picking up where you left off after hours or days
- [ ] **Session handoff notes** — export a session's summary timeline as markdown, so you can hand context to a colleague or your future self
- [ ] **Diff-aware summaries** — highlight what changed since your last check-in, not just the current state. Helps when reviewing multiple parallel sessions
- [ ] **Smarter idle detection** — distinguish "waiting for user input" from "long-running process still working"
- [ ] **Keyboard shortcut to cycle through errors** — jump directly to panes with RED status
- [ ] **Per-pane cost breakdown** — see token spend per session, not just the total

### Ideas we're considering (no promises)
- [ ] Detect specific tools (Claude Code, Codex, vim, cargo) and tailor summaries to their output patterns
- [ ] SSH-aware summarization — detect remote sessions and include host info
- [ ] Budget alerts — warn when token spend exceeds a threshold
- [ ] Configurable sidebar position (left or right) and width

### Out of scope
These are real problems, but they need different tools:
- **Git worktree isolation per agent** — use [git worktrees](https://git-scm.com/docs/git-worktree) or tools like [Parallel Code](https://dev.to/johannesjo/why-multitasking-with-ai-coding-agents-breaks-down-and-how-i-fixed-it-2lm0)
- **Background/cloud agent execution** — requires a daemon architecture, not a terminal plugin
- **Agent orchestration** — Muxwise observes sessions, it doesn't control them

We're building what's useful, not what sounds impressive. If you have ideas, [open an issue](https://github.com/caophuong2012/muxwise/issues).

## Development

```bash
git clone https://github.com/caophuong2012/muxwise.git
cd muxwise
cargo xtask run          # debug build
cargo xtask test         # run tests
```

For more build commands, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgments

Muxwise is built on top of [Zellij](https://github.com/zellij-org/zellij), a terminal multiplexer created by the Zellij Contributors. All original Zellij code retains its copyright and license. The Session Intelligence plugin (`default-plugins/session-intelligence/`) is an original addition.

## License

MIT — see [LICENSE.md](LICENSE.md)

```
Original Zellij code: Copyright (c) 2020 Zellij Contributors
Session Intelligence plugin: Copyright (c) 2026 Muxwise Contributors
```

Both are licensed under the MIT License.
