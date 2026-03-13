use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sanitize;
use crate::state::{PaneData, PaneStatus, PaneSummary, PluginState};

/// Current persistence format version.
const PERSISTENCE_VERSION: u32 = 1;

/// Subdirectory under `~/.local/share/zellij/` where state files are stored.
const STATE_DIR: &str = "session-intelligence";

/// Serializable representation of the full plugin state for JSON persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    /// Schema version for forward-compatibility checks.
    pub version: u32,
    /// Whether the sidebar was visible when state was saved.
    pub sidebar_visible: bool,
    /// Per-pane persisted data, keyed by "{pane_id}_{is_plugin}".
    pub panes: HashMap<String, PersistedPaneData>,
}

/// Serializable per-pane data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPaneData {
    /// Display name of the pane.
    pub name: String,
    /// AI-generated summary text, if available.
    pub summary_text: Option<String>,
    /// Status string: "GREEN", "YELLOW", or "RED".
    pub status: Option<String>,
    /// Timestamp of when the summary was generated.
    pub generated_at: Option<String>,
    /// Hash of the last captured scrollback (for change detection).
    pub last_scrollback_hash: u64,
}

/// Build the pane key string from a pane_id and is_plugin flag.
fn pane_key(pane_id: u32, is_plugin: bool) -> String {
    format!("{}_{}", pane_id, is_plugin)
}

/// Parse a pane key string back into (pane_id, is_plugin).
/// Returns None if the key doesn't match the expected format.
fn parse_pane_key(key: &str) -> Option<(u32, bool)> {
    let parts: Vec<&str> = key.rsplitn(2, '_').collect();
    if parts.len() != 2 {
        return None;
    }
    // rsplitn reverses the order: parts[0] is the is_plugin part, parts[1] is the pane_id
    let is_plugin = match parts[0] {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    let pane_id = parts[1].parse::<u32>().ok()?;
    Some((pane_id, is_plugin))
}

/// Determine the directory path for storing persistence files.
///
/// Returns `~/.local/share/zellij/session-intelligence/` or falls back
/// to `/tmp/zellij-session-intelligence/` if HOME is not set.
fn state_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("zellij")
        .join(STATE_DIR)
}

/// Compute the full path for a session's JSON state file.
fn state_file_path(session_name: &str) -> std::path::PathBuf {
    state_dir().join(format!("{}.json", session_name))
}

/// Directory for scrollback snapshots for a given session.
fn scrollback_dir(session_name: &str) -> std::path::PathBuf {
    state_dir().join("scrollback").join(session_name)
}

/// File path for a single pane's scrollback snapshot.
fn scrollback_file_path(session_name: &str, pane_id: u32, is_plugin: bool) -> std::path::PathBuf {
    scrollback_dir(session_name).join(format!("{}_{}.txt", pane_id, is_plugin))
}

/// Save a scrollback snapshot for a pane.
///
/// The scrollback is sanitized before writing to disk to prevent
/// sensitive data from being persisted in plain text files.
pub fn save_scrollback(session_name: &str, pane_id: u32, is_plugin: bool, scrollback: &str) {
    if session_name.is_empty() || scrollback.is_empty() {
        return;
    }

    let dir = scrollback_dir(session_name);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "session-intelligence: persistence: failed to create scrollback dir {:?}: {}",
            dir, e
        );
        return;
    }

    // Sanitize before writing to disk.
    let sanitized = sanitize::sanitize_scrollback(scrollback);

    let path = scrollback_file_path(session_name, pane_id, is_plugin);
    if let Err(e) = std::fs::write(&path, &sanitized) {
        eprintln!(
            "session-intelligence: persistence: failed to write scrollback {:?}: {}",
            path, e
        );
    }
}

/// Load a scrollback snapshot for a pane.
///
/// Returns None if the file doesn't exist or can't be read.
pub fn load_scrollback(session_name: &str, pane_id: u32, is_plugin: bool) -> Option<String> {
    if session_name.is_empty() {
        return None;
    }

    let path = scrollback_file_path(session_name, pane_id, is_plugin);
    match std::fs::read_to_string(&path) {
        Ok(content) if !content.is_empty() => Some(content),
        Ok(_) => None,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "session-intelligence: persistence: failed to read scrollback {:?}: {}",
                    path, e
                );
            }
            None
        },
    }
}

/// Save the current plugin state to a JSON file on disk.
///
/// Converts `PluginState` into `PersistedState`, serializes it as JSON,
/// and writes it to `~/.local/share/zellij/session-intelligence/{session_name}.json`.
///
/// All errors are logged via `eprintln!` and never cause a crash.
pub fn save_state(state: &PluginState, session_name: &str) {
    if session_name.is_empty() {
        eprintln!("session-intelligence: persistence: skipping save, no session name");
        return;
    }

    // Convert PluginState to PersistedState.
    let mut panes = HashMap::new();
    for (&(pane_id, is_plugin), pane_data) in &state.panes {
        let key = pane_key(pane_id, is_plugin);
        let (summary_text, status, generated_at) = match &pane_data.summary {
            Some(summary) => (
                Some(summary.text.clone()),
                Some(summary.status.to_str().to_string()),
                Some(summary.generated_at.clone()),
            ),
            None => (None, None, None),
        };

        panes.insert(
            key,
            PersistedPaneData {
                name: pane_data.name.clone(),
                summary_text,
                status,
                generated_at,
                last_scrollback_hash: pane_data.last_scrollback_hash,
            },
        );
    }

    let persisted = PersistedState {
        version: PERSISTENCE_VERSION,
        sidebar_visible: state.sidebar_visible,
        panes,
    };

    // Serialize to JSON.
    let json = match serde_json::to_string_pretty(&persisted) {
        Ok(j) => j,
        Err(e) => {
            eprintln!(
                "session-intelligence: persistence: failed to serialize state: {}",
                e
            );
            return;
        },
    };

    // Ensure the directory exists.
    let dir = state_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "session-intelligence: persistence: failed to create directory {:?}: {}",
            dir, e
        );
        return;
    }

    // Write the JSON file.
    let path = state_file_path(session_name);
    if let Err(e) = std::fs::write(&path, json) {
        eprintln!(
            "session-intelligence: persistence: failed to write {:?}: {}",
            path, e
        );
    }
}

/// Load persisted state from disk.
///
/// Reads `~/.local/share/zellij/session-intelligence/{session_name}.json`,
/// deserializes the JSON, and returns the `PersistedState`.
///
/// Returns `None` if the file doesn't exist or cannot be parsed.
/// All errors are logged via `eprintln!` and never cause a crash.
pub fn load_state(session_name: &str) -> Option<PersistedState> {
    if session_name.is_empty() {
        return None;
    }

    let path = state_file_path(session_name);

    let json = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "session-intelligence: persistence: failed to read {:?}: {}",
                    path, e
                );
            }
            return None;
        },
    };

    match serde_json::from_str::<PersistedState>(&json) {
        Ok(persisted) => {
            eprintln!(
                "session-intelligence: persistence: loaded state from {:?} (version {})",
                path, persisted.version
            );
            Some(persisted)
        },
        Err(e) => {
            eprintln!(
                "session-intelligence: persistence: failed to parse {:?}: {}",
                path, e
            );
            None
        },
    }
}

/// Restore persisted state into the live `PluginState`.
///
/// Restores sidebar visibility and per-pane summaries/hashes. Only restores
/// data for panes that currently exist in the plugin state (panes are matched
/// by their key). Panes in the persisted data that don't exist in the live
/// state are added so that summaries survive across pane re-creation at the
/// same ID.
pub fn restore_into(persisted: &PersistedState, state: &mut PluginState) {
    // Restore sidebar visibility.
    state.sidebar_visible = persisted.sidebar_visible;

    // Restore per-pane data.
    for (key, persisted_pane) in &persisted.panes {
        let (pane_id, is_plugin) = match parse_pane_key(key) {
            Some(parsed) => parsed,
            None => {
                eprintln!(
                    "session-intelligence: persistence: skipping invalid pane key '{}'",
                    key
                );
                continue;
            },
        };

        let pane_key = (pane_id, is_plugin);

        // Get or create the pane entry.
        let pane_data = state.panes.entry(pane_key).or_insert_with(|| PaneData {
            name: persisted_pane.name.clone(),
            is_plugin,
            tab_index: 0,
            last_scrollback_hash: 0,
            summary: None,
            last_summarized_at: 0.0,
            last_scrollback: None,
            cwd: None,
        });

        // Restore the scrollback hash.
        pane_data.last_scrollback_hash = persisted_pane.last_scrollback_hash;

        // Restore scrollback snapshot from disk.
        pane_data.last_scrollback = load_scrollback(&state.session_name, pane_id, is_plugin);

        // Restore the summary if one was persisted.
        if let Some(ref summary_text) = persisted_pane.summary_text {
            let status = persisted_pane
                .status
                .as_deref()
                .map(PaneStatus::from_str)
                .unwrap_or_default();
            let generated_at = persisted_pane
                .generated_at
                .clone()
                .unwrap_or_default();

            pane_data.summary = Some(PaneSummary {
                text: summary_text.clone(),
                status,
                generated_at,
                is_stale: false,
            });
        }
    }

    let scrollback_count = state.panes.values()
        .filter(|p| p.last_scrollback.is_some())
        .count();
    eprintln!(
        "session-intelligence: persistence: restored {} pane(s), {} with scrollback, sidebar_visible={}",
        persisted.panes.len(),
        scrollback_count,
        persisted.sidebar_visible
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_key_roundtrip() {
        let key = pane_key(42, false);
        assert_eq!(key, "42_false");
        let parsed = parse_pane_key(&key);
        assert_eq!(parsed, Some((42, false)));

        let key2 = pane_key(7, true);
        assert_eq!(key2, "7_true");
        let parsed2 = parse_pane_key(&key2);
        assert_eq!(parsed2, Some((7, true)));
    }

    #[test]
    fn test_parse_pane_key_invalid() {
        assert_eq!(parse_pane_key("invalid"), None);
        assert_eq!(parse_pane_key("_true"), None);
        assert_eq!(parse_pane_key("abc_true"), None);
        assert_eq!(parse_pane_key("42_maybe"), None);
    }

    #[test]
    fn test_persisted_state_serialization() {
        let mut panes = HashMap::new();
        panes.insert(
            "1_false".to_string(),
            PersistedPaneData {
                name: "test-pane".to_string(),
                summary_text: Some("All good".to_string()),
                status: Some("GREEN".to_string()),
                generated_at: Some("2024-01-01T00:00:00Z".to_string()),
                last_scrollback_hash: 12345,
            },
        );

        let state = PersistedState {
            version: 1,
            sidebar_visible: true,
            panes,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PersistedState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, 1);
        assert!(deserialized.sidebar_visible);
        assert_eq!(deserialized.panes.len(), 1);

        let pane = deserialized.panes.get("1_false").unwrap();
        assert_eq!(pane.name, "test-pane");
        assert_eq!(pane.summary_text, Some("All good".to_string()));
        assert_eq!(pane.status, Some("GREEN".to_string()));
        assert_eq!(pane.last_scrollback_hash, 12345);
    }

    #[test]
    fn test_pane_status_roundtrip() {
        assert_eq!(PaneStatus::from_str(PaneStatus::Active.to_str()), PaneStatus::Active);
        assert_eq!(PaneStatus::from_str(PaneStatus::Waiting.to_str()), PaneStatus::Waiting);
        assert_eq!(PaneStatus::from_str(PaneStatus::Error.to_str()), PaneStatus::Error);
        assert_eq!(PaneStatus::from_str("UNKNOWN"), PaneStatus::Waiting);
    }
}
