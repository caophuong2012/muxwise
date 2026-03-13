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

/// Detect the working directory from pane title and scrollback.
///
/// Tries multiple strategies:
/// 1. Shell prompt patterns like `user@host:/path$` or `user@host ~/path`
/// 2. Claude Code header showing repo path (e.g., `~/openclaw`)
/// 3. Common prompt formats: `path $`, `path %`, `path >`
///
/// Returns the shortened path (last 2 components) or None.
pub fn detect_cwd(title: &str, scrollback: &str) -> Option<String> {
    // Strategy 1: Parse from pane title (shells often set title to "user@host:path")
    if let Some(cwd) = parse_cwd_from_title(title) {
        return Some(shorten_path(&cwd));
    }

    // Strategy 2: Scan last ~20 lines of scrollback for prompt patterns.
    let lines: Vec<&str> = scrollback.lines().collect();
    let start = lines.len().saturating_sub(20);
    for line in lines[start..].iter().rev() {
        if let Some(cwd) = parse_cwd_from_prompt(line) {
            return Some(shorten_path(&cwd));
        }
    }

    None
}

/// Extract CWD from a terminal title string.
/// Common formats: "user@host:~/path", "~/path", "/home/user/path"
fn parse_cwd_from_title(title: &str) -> Option<String> {
    // "user@host:~/path" or "user@host:/path"
    if let Some(pos) = title.find(':') {
        let after = title[pos + 1..].trim();
        if after.starts_with('/') || after.starts_with('~') {
            return Some(after.to_string());
        }
    }
    // Title is just a path
    let trimmed = title.trim();
    if trimmed.starts_with('/') || trimmed.starts_with("~/") {
        return Some(trimmed.to_string());
    }
    None
}

/// Extract CWD from a shell prompt line.
fn parse_cwd_from_prompt(line: &str) -> Option<String> {
    let trimmed = line.trim();

    // "user@host ~/path" or "user@host /path" followed by prompt char
    if trimmed.contains('@') {
        // Split on spaces, look for a path-like token
        for token in trimmed.split_whitespace() {
            if (token.starts_with('/') || token.starts_with("~/")) && !token.contains('@') {
                return Some(token.to_string());
            }
        }
    }

    // "~/path $" or "/path >" or "/path %"
    if trimmed.ends_with('$') || trimmed.ends_with('>') || trimmed.ends_with('%') {
        let without_prompt = trimmed[..trimmed.len() - 1].trim();
        // Last token before prompt might be the path
        if let Some(last) = without_prompt.split_whitespace().last() {
            if last.starts_with('/') || last.starts_with("~/") {
                return Some(last.to_string());
            }
        }
    }

    None
}

/// Shorten a path to the last 2 components for display.
/// e.g., "/home/user/Work/BoldLab/muxwise" -> "BoldLab/muxwise"
/// e.g., "~/Work/project" -> "Work/project"
fn shorten_path(path: &str) -> String {
    let clean = path.trim_end_matches('/');
    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 2 {
        return clean.to_string();
    }
    let last_two = &parts[parts.len() - 2..];
    last_two.join("/")
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

    #[test]
    fn detect_cwd_from_title_user_at_host() {
        let cwd = detect_cwd("user@host:~/Work/project", "");
        assert_eq!(cwd, Some("Work/project".to_string()));
    }

    #[test]
    fn detect_cwd_from_title_path_only() {
        let cwd = detect_cwd("~/myproject", "");
        assert_eq!(cwd, Some("~/myproject".to_string()));
    }

    #[test]
    fn detect_cwd_from_prompt_line() {
        let scrollback = "some output\nuser@host ~/Work/muxwise\n$ ";
        let cwd = detect_cwd("Claude Code", scrollback);
        assert_eq!(cwd, Some("Work/muxwise".to_string()));
    }

    #[test]
    fn shorten_long_path() {
        assert_eq!(shorten_path("/home/user/Work/BoldLab/muxwise"), "BoldLab/muxwise");
    }

    #[test]
    fn shorten_short_path() {
        assert_eq!(shorten_path("~/project"), "~/project");
    }

    #[test]
    fn detect_cwd_none_for_unrecognized() {
        assert_eq!(detect_cwd("Claude Code", "hello world"), None);
    }
}
