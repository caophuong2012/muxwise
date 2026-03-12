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

Download a prebuilt binary from [Releases](https://github.com/caophuong2012/muxwise/releases) for your platform, or build from source:

```bash
git clone https://github.com/caophuong2012/muxwise.git
cd muxwise
cargo xtask build --release
```

The Session Intelligence sidebar is included in the default layout — just launch and it's there.

## Configuration

Set your AI API key in `~/.config/zellij/config.kdl`:

```kdl
plugins {
    session-intelligence location="zellij:session-intelligence" {
        ai_api_key "your-api-key-here"
        ai_provider "anthropic"          // or "openai"
        summarization_interval "60"      // seconds between scans
        buffer_size "2000"               // max scrollback lines
        cooldown "30"                    // min seconds between re-summarizing same pane
    }
}
```

For general Zellij configuration, see the [Zellij Configuration Documentation](https://zellij.dev/documentation/configuration.html).

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
