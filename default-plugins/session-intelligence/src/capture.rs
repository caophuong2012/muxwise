use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use zellij_tile::prelude::*;

/// Fetch the scrollback text for a given pane.
///
/// Calls the Zellij `get_pane_scrollback` API with `full_scrollback = true`,
/// then truncates the result to at most `buffer_size` lines (keeping the most
/// recent lines, i.e., the tail of the output).
///
/// Returns an empty string if the pane cannot be read or an error occurs.
pub fn fetch_scrollback(pane_id: u32, is_plugin: bool, buffer_size: usize) -> String {
    let zellij_pane_id = if is_plugin {
        PaneId::Plugin(pane_id)
    } else {
        PaneId::Terminal(pane_id)
    };

    match get_pane_scrollback(zellij_pane_id, true) {
        Ok(pane_contents) => {
            // Combine all lines: lines above viewport + viewport + lines below viewport.
            let mut all_lines: Vec<String> = Vec::new();
            all_lines.extend(pane_contents.lines_above_viewport);
            all_lines.extend(pane_contents.viewport);
            all_lines.extend(pane_contents.lines_below_viewport);

            // Truncate to the most recent `buffer_size` lines.
            if all_lines.len() > buffer_size {
                let start = all_lines.len() - buffer_size;
                all_lines[start..].join("\n")
            } else {
                all_lines.join("\n")
            }
        },
        Err(e) => {
            eprintln!(
                "session-intelligence: failed to fetch scrollback for pane {}: {}",
                pane_id, e
            );
            String::new()
        },
    }
}

/// Compute a fast hash of the scrollback text.
///
/// Uses `DefaultHasher` (SipHash) for a quick content-change check.
/// Returns 0 for empty input so that the initial default hash (0) matches
/// empty panes and avoids spurious queuing.
pub fn hash_scrollback(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_empty_is_zero() {
        assert_eq!(hash_scrollback(""), 0);
    }

    #[test]
    fn hash_deterministic() {
        let text = "hello world\nline two";
        let h1 = hash_scrollback(text);
        let h2 = hash_scrollback(text);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different_for_different_input() {
        let h1 = hash_scrollback("hello");
        let h2 = hash_scrollback("world");
        assert_ne!(h1, h2);
    }
}
