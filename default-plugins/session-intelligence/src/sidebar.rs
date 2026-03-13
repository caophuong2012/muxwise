use crate::state::{PaneData, PaneStatus, PluginState};
use zellij_tile::prelude::*;

/// Target sidebar width in columns.
const SIDEBAR_WIDTH: usize = 30;

/// Unicode left-edge status block character.
const STATUS_BLOCK: &str = "\u{258c}"; // ▌

/// Indent prefix for summary and timestamp lines (2 spaces).
const INDENT: &str = "  ";

/// Render the empty state message when no panes are tracked yet.
fn render_empty_state(row: usize, width: usize, has_api_key: bool) {
    let mut r = row;
    let w = width.saturating_sub(1);

    let msg1 = "No panes detected yet.";
    print_text_with_coordinates(Text::new(msg1).dim_all(), 1, r, Some(w), Some(1));
    r += 2;

    if !has_api_key {
        let lines = [
            "Setup:",
            "  1. Get an API key from:",
            "     - console.anthropic.com",
            "     - platform.openai.com",
            "     - openrouter.ai",
            "  2. Add to config:",
            "     ~/.config/zellij/config.kdl",
            "",
            "  plugins {",
            "    session-intelligence ... {",
            "      ai_api_key \"your-key\"",
            "    }",
            "  }",
        ];
        for line in &lines {
            let text = Text::new(line).dim_all();
            print_text_with_coordinates(text, 1, r, Some(w), Some(1));
            r += 1;
        }
    }
}

/// Hard-wrap a string to fit within `max_width` columns.
///
/// Returns a Vec of lines, each no longer than `max_width` characters.
/// Wraps on word boundaries when possible, otherwise breaks mid-word.
fn hard_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();

    for input_line in text.lines() {
        let words: Vec<&str> = input_line.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        for word in words {
            if current_line.is_empty() {
                // First word on the line -- if it exceeds max_width, break it.
                if word.chars().count() > max_width {
                    let mut chars = word.chars();
                    while chars.as_str().len() > 0 {
                        let chunk: String = chars.by_ref().take(max_width).collect();
                        if chunk.is_empty() {
                            break;
                        }
                        lines.push(chunk);
                    }
                } else {
                    current_line = word.to_string();
                }
            } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                // Start new line with this word, breaking if needed.
                if word.chars().count() > max_width {
                    let mut chars = word.chars();
                    while chars.as_str().len() > 0 {
                        let chunk: String = chars.by_ref().take(max_width).collect();
                        if chunk.is_empty() {
                            break;
                        }
                        lines.push(chunk);
                    }
                    current_line = String::new();
                } else {
                    current_line = word.to_string();
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Render a single pane entry and return the number of rows consumed.
///
/// Layout per entry:
///   Row 0:  ▌ <pane_name>          (status color block + bold name)
///   Row 1+: <2-space indent><summary line>  (normal weight, wrapped)
///   Row N:  <2-space indent><timestamp>     (dim text)
///   Row N+1: (blank line separator)
///
/// Panes without summaries are shown with a dim placeholder line.
fn render_pane_entry(
    pane_data: &PaneData,
    start_row: usize,
    width: usize,
    max_rows: usize,
    has_api_key: bool,
) -> usize {
    if max_rows == 0 || width == 0 {
        return 0;
    }

    let mut rows_used: usize = 0;

    // Determine summary and status, falling back to defaults for panes without summaries.
    let no_key_msg = "Set ai_api_key in ~/.config/zellij/config.kdl";
    let awaiting_msg = "Awaiting summary...";
    let (summary_text, status, timestamp, is_stale, has_summary) = match &pane_data.summary {
        Some(summary) => (
            summary.text.as_str(),
            &summary.status,
            summary.generated_at.as_str(),
            summary.is_stale,
            true,
        ),
        None => {
            let msg = if has_api_key { awaiting_msg } else { no_key_msg };
            (msg, &PaneStatus::Waiting, "", false, false)
        },
    };

    // -- Row 0: Status block + pane name --
    // Build the header line: "▌ <name>"
    let status_char_len = STATUS_BLOCK.chars().count(); // 1 char for ▌
    let separator = " ";
    let name_max = width.saturating_sub(status_char_len + separator.len());
    let display_name: String = if pane_data.name.chars().count() > name_max {
        if name_max > 3 {
            let truncated: String = pane_data.name.chars().take(name_max - 3).collect();
            format!("{}...", truncated)
        } else {
            pane_data.name.chars().take(name_max).collect()
        }
    } else {
        pane_data.name.clone()
    };

    let header_line = format!("{}{}{}", STATUS_BLOCK, separator, display_name);

    // Color the status block character according to pane status.
    // When stale, override to yellow/accent to indicate staleness.
    // When no summary yet, show dim.
    let header_text = if !has_summary {
        Text::new(&header_line)
            .color_range(3, 0..status_char_len) // yellow for pending
            .dim_all()
    } else if is_stale {
        Text::new(&header_line)
            .color_range(3, 0..status_char_len) // yellow for stale
            .dim_all()
    } else {
        match status {
            PaneStatus::Active => Text::new(&header_line)
                .color_range(7, 0..status_char_len) // success/green for the block
                .selected(), // bold the entire line (pane name)
            PaneStatus::Waiting => Text::new(&header_line)
                .color_range(3, 0..status_char_len) // accent/yellow for the block
                .selected(),
            PaneStatus::Error => Text::new(&header_line)
                .color_range(6, 0..status_char_len) // error/red for the block
                .selected(),
        }
    };

    let row = start_row + rows_used;
    if row >= start_row + max_rows {
        return rows_used;
    }
    print_text_with_coordinates(header_text, 0, row, Some(width), Some(1));
    rows_used += 1;

    // -- CWD line (dim, indented) --
    if let Some(ref cwd) = pane_data.cwd {
        let row = start_row + rows_used;
        if row >= start_row + max_rows {
            return rows_used;
        }
        let cwd_line = format!("{}{}", INDENT, cwd);
        let cwd_text = Text::new(&cwd_line).color_range(3, ..).dim_all();
        print_text_with_coordinates(cwd_text, 0, row, Some(width), Some(1));
        rows_used += 1;
    }

    // -- Summary lines (2-3 lines, indented, normal weight; dim if stale or pending) --
    let summary_width = width.saturating_sub(INDENT.chars().count());
    let wrapped_lines = hard_wrap(summary_text, summary_width);

    for line in &wrapped_lines {
        let row = start_row + rows_used;
        if row >= start_row + max_rows {
            return rows_used;
        }
        let indented = format!("{}{}", INDENT, line);
        let line_text = if is_stale || !has_summary {
            Text::new(&indented).dim_all()
        } else {
            Text::new(&indented)
        };
        print_text_with_coordinates(line_text, 0, row, Some(width), Some(1));
        rows_used += 1;
    }

    // -- Timestamp line (dim text, indented) --
    if !timestamp.is_empty() {
        let row = start_row + rows_used;
        if row >= start_row + max_rows {
            return rows_used;
        }
        let ts_line = format!("{}{}", INDENT, timestamp);
        let ts_text = Text::new(&ts_line).dim_all();
        print_text_with_coordinates(ts_text, 0, row, Some(width), Some(1));
        rows_used += 1;
    }

    // -- Blank line separator --
    let row = start_row + rows_used;
    if row < start_row + max_rows {
        let blank = Text::new("");
        print_text_with_coordinates(blank, 0, row, Some(width), Some(1));
        rows_used += 1;
    }

    rows_used
}

/// Render the sidebar panel on the left side of the plugin pane.
///
/// Displays a header, separator, and either:
/// - Rich pane entries with status colors, summaries, and timestamps
/// - An empty state message when no summaries exist
pub fn render_sidebar(state: &mut PluginState, rows: usize, cols: usize) {
    // Reset the click map for this render cycle.
    state.click_map.clear();
    state.click_map.resize(rows, None);

    if !state.sidebar_visible {
        return;
    }

    let width = cols.min(SIDEBAR_WIDTH);
    if width == 0 || rows == 0 {
        return;
    }

    // -- Header --
    let header = "Muxwise";
    let header_text = Text::new(header).selected();
    print_text_with_coordinates(header_text, 0, 0, Some(width), Some(1));

    // -- Separator line --
    let separator: String = "\u{2500}".repeat(width);
    let sep_text = Text::new(&separator);
    print_text_with_coordinates(sep_text, 0, 1, Some(width), Some(1));

    // -- Pane list or empty state --
    let content_start_row: usize = 2;

    // Collect and sort non-plugin pane entries for the active tab only.
    let active_tab = state.active_tab_index;
    let mut entries: Vec<_> = state
        .panes
        .iter()
        .filter(|((_, is_plugin), pane_data)| !is_plugin && pane_data.tab_index == active_tab)
        .collect();
    entries.sort_by_key(|((id, is_plugin), _)| (*id, *is_plugin));

    let has_api_key = state.config.api_key.is_some();

    if entries.is_empty() {
        render_empty_state(content_start_row, width, has_api_key);
    } else {

        // Render each pane entry, tracking the current row position.
        let mut current_row = content_start_row;
        let mut entries_to_skip = state.scroll_offset;

        for ((id, is_plugin), pane_data) in &entries {
            // Apply scroll offset by skipping entries.
            if entries_to_skip > 0 {
                entries_to_skip -= 1;
                continue;
            }

            let remaining_rows = rows.saturating_sub(current_row);
            if remaining_rows == 0 {
                break;
            }

            let rows_consumed =
                render_pane_entry(pane_data, current_row, width, remaining_rows, has_api_key);

            // Populate click_map: all rows of this entry map to this pane.
            for r in current_row..current_row + rows_consumed {
                if r < state.click_map.len() {
                    state.click_map[r] = Some((*id, *is_plugin));
                }
            }

            current_row += rows_consumed;
        }
    }

    // -- Diagnostic footer at the bottom --
    render_diagnostics(state, rows, width);
}

/// Format a token count for compact display (e.g., 1234 -> "1.2k").
fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

/// Render diagnostic info at the bottom of the sidebar.
fn render_diagnostics(state: &PluginState, rows: usize, width: usize) {
    if rows < 4 || width == 0 {
        return;
    }

    let sep_row = rows.saturating_sub(3);
    let separator: String = "\u{2500}".repeat(width);
    let sep_text = Text::new(&separator).dim_all();
    print_text_with_coordinates(sep_text, 0, sep_row, Some(width), Some(1));

    // Line 1: status message (scan status, errors, etc.)
    if !state.last_status_msg.is_empty() {
        let status_display: String = state.last_status_msg.chars().take(width).collect();
        let status_text = Text::new(&status_display).dim_all();
        print_text_with_coordinates(status_text, 0, sep_row + 1, Some(width), Some(1));
    }

    // Line 2: token usage
    let total_tokens = state.total_input_tokens + state.total_output_tokens;
    let stats = if total_tokens > 0 {
        format!("tokens: {}", format_tokens(total_tokens))
    } else {
        "tokens: 0".to_string()
    };
    let stats_text = Text::new(&stats).dim_all();
    print_text_with_coordinates(stats_text, 0, sep_row + 2, Some(width), Some(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_wrap_short_text() {
        let lines = hard_wrap("hello world", 20);
        assert_eq!(lines, vec!["hello world"]);
    }

    #[test]
    fn test_hard_wrap_exact_width() {
        let lines = hard_wrap("hello", 5);
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn test_hard_wrap_needs_wrapping() {
        let lines = hard_wrap("hello world foo", 11);
        assert_eq!(lines, vec!["hello world", "foo"]);
    }

    #[test]
    fn test_hard_wrap_long_word() {
        let lines = hard_wrap("abcdefghij", 5);
        assert_eq!(lines, vec!["abcde", "fghij"]);
    }

    #[test]
    fn test_hard_wrap_empty() {
        let lines = hard_wrap("", 10);
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn test_hard_wrap_zero_width() {
        let lines = hard_wrap("hello", 0);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_hard_wrap_multiline_input() {
        let lines = hard_wrap("line one\nline two", 20);
        assert_eq!(lines, vec!["line one", "line two"]);
    }
}
