use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use zellij_tile::prelude::*;

/// Default summarization interval in seconds.
const DEFAULT_SUMMARIZATION_INTERVAL_SECS: f64 = 60.0;

/// Default buffer size in lines.
const DEFAULT_BUFFER_SIZE_LINES: usize = 2000;

/// Minimum seconds between re-summarizing the same pane, even if content changed.
const DEFAULT_COOLDOWN_SECS: f64 = 30.0;

/// Which AI provider to use for summarization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiProvider {
    Anthropic,
    OpenAi,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::Anthropic
    }
}

/// Plugin configuration parsed from KDL config values.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// API key for the AI service. None if not configured.
    pub api_key: Option<String>,
    /// Which AI provider to use.
    pub ai_provider: AiProvider,
    /// How often (in seconds) to trigger summarization.
    pub summarization_interval_secs: f64,
    /// Maximum number of scrollback lines to capture per pane.
    pub buffer_size_lines: usize,
    /// Minimum seconds between re-summarizing the same pane.
    pub cooldown_secs: f64,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            ai_provider: AiProvider::default(),
            summarization_interval_secs: DEFAULT_SUMMARIZATION_INTERVAL_SECS,
            buffer_size_lines: DEFAULT_BUFFER_SIZE_LINES,
            cooldown_secs: DEFAULT_COOLDOWN_SECS,
        }
    }
}

impl PluginConfig {
    /// Parse plugin configuration from the KDL config BTreeMap.
    ///
    /// Expected keys:
    /// - `ai_api_key`: API key string (optional, warns if missing)
    /// - `summarization_interval`: interval in seconds as a number (optional, defaults to 120)
    /// - `buffer_size`: max scrollback lines as an integer (optional, defaults to 2000)
    pub fn from_btree(config: &BTreeMap<String, String>) -> Self {
        let api_key = config.get("ai_api_key").cloned().filter(|k| !k.is_empty());

        if api_key.is_none() {
            eprintln!(
                "session-intelligence: warning: no ai_api_key configured; \
                 AI summarization will be unavailable"
            );
        }

        let ai_provider = config
            .get("ai_provider")
            .map(|v| match v.to_lowercase().as_str() {
                "openai" | "open_ai" | "open-ai" => AiProvider::OpenAi,
                _ => AiProvider::Anthropic,
            })
            .unwrap_or(AiProvider::Anthropic);

        let summarization_interval_secs = config
            .get("summarization_interval")
            .and_then(|v| match v.parse::<f64>() {
                Ok(val) if val > 0.0 => Some(val),
                Ok(val) => {
                    eprintln!(
                        "session-intelligence: warning: invalid summarization_interval '{}', \
                         using default {}s",
                        val, DEFAULT_SUMMARIZATION_INTERVAL_SECS
                    );
                    None
                },
                Err(e) => {
                    eprintln!(
                        "session-intelligence: warning: failed to parse summarization_interval \
                         '{}': {}, using default {}s",
                        v, e, DEFAULT_SUMMARIZATION_INTERVAL_SECS
                    );
                    None
                },
            })
            .unwrap_or(DEFAULT_SUMMARIZATION_INTERVAL_SECS);

        let buffer_size_lines = config
            .get("buffer_size")
            .and_then(|v| match v.parse::<usize>() {
                Ok(val) if val > 0 => Some(val),
                Ok(val) => {
                    eprintln!(
                        "session-intelligence: warning: invalid buffer_size '{}', \
                         using default {}",
                        val, DEFAULT_BUFFER_SIZE_LINES
                    );
                    None
                },
                Err(e) => {
                    eprintln!(
                        "session-intelligence: warning: failed to parse buffer_size \
                         '{}': {}, using default {}",
                        v, e, DEFAULT_BUFFER_SIZE_LINES
                    );
                    None
                },
            })
            .unwrap_or(DEFAULT_BUFFER_SIZE_LINES);

        let cooldown_secs = config
            .get("cooldown")
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| *v >= 0.0)
            .unwrap_or(DEFAULT_COOLDOWN_SECS);

        Self {
            api_key,
            ai_provider,
            summarization_interval_secs,
            buffer_size_lines,
            cooldown_secs,
        }
    }
}

/// Status indicator for a pane's health/state as determined by the AI summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneStatus {
    /// Everything looks normal / actively working (green).
    Active,
    /// Waiting for input or idle (yellow).
    Waiting,
    /// Error or problem detected (red).
    Error,
}

impl PaneStatus {
    /// Convert to a string representation for persistence.
    pub fn to_str(&self) -> &str {
        match self {
            PaneStatus::Active => "GREEN",
            PaneStatus::Waiting => "YELLOW",
            PaneStatus::Error => "RED",
        }
    }

    /// Parse from a string representation. Returns default (Waiting) for unknown values.
    pub fn from_str(s: &str) -> PaneStatus {
        match s {
            "GREEN" => PaneStatus::Active,
            "YELLOW" => PaneStatus::Waiting,
            "RED" => PaneStatus::Error,
            _ => PaneStatus::Waiting,
        }
    }
}

impl Default for PaneStatus {
    fn default() -> Self {
        PaneStatus::Waiting
    }
}

/// AI-generated summary for a single pane.
#[derive(Debug, Clone)]
pub struct PaneSummary {
    /// The summary text (2-3 lines).
    pub text: String,
    /// The status color derived from the AI response.
    pub status: PaneStatus,
    /// Timestamp string of when this summary was generated.
    pub generated_at: String,
    /// Whether this summary is stale (scrollback has changed since generation).
    pub is_stale: bool,
}

/// Per-pane tracking data.
#[derive(Debug, Default, Clone)]
pub struct PaneData {
    pub name: String,
    pub is_plugin: bool,
    /// The tab index this pane belongs to.
    pub tab_index: usize,
    /// Hash of the last captured scrollback content. Used for change detection
    /// so that only panes whose output has changed are queued for summarization.
    /// Defaults to 0 (matching the hash of empty content).
    pub last_scrollback_hash: u64,
    /// AI-generated summary for this pane, if available.
    pub summary: Option<PaneSummary>,
    /// Elapsed time (in seconds) when the last summary was generated.
    /// Used for per-pane cooldown to avoid re-summarizing too frequently.
    pub last_summarized_at: f64,
}

/// Central plugin state. All state flows through this struct.
#[derive(Default)]
pub struct PluginState {
    /// Plugin configuration loaded from KDL.
    pub config: PluginConfig,
    /// Whether the sidebar panel is visible.
    pub sidebar_visible: bool,
    /// Tracked panes, keyed by (pane_id, is_plugin).
    pub panes: HashMap<(u32, bool), PaneData>,
    /// Current scroll offset for the sidebar pane list.
    pub scroll_offset: usize,
    /// Number of rows available for rendering (from last render call).
    pub rows: usize,
    /// Number of columns available for rendering (from last render call).
    pub cols: usize,
    /// Queue of panes whose scrollback has changed and need summarization.
    /// Each entry is (pane_id, is_plugin).
    pub summarization_queue: VecDeque<(u32, bool)>,
    /// Currently pending summarization request, if any.
    /// Blocks further requests until the current one completes.
    /// Value is (pane_id, is_plugin).
    pub pending_request: Option<(u32, bool)>,
    /// Mapping of sidebar rows to pane IDs for click-to-navigate.
    /// Built during render, maps row index -> (pane_id, is_plugin).
    pub click_map: Vec<Option<(u32, bool)>>,
    /// Session name discovered at runtime from SessionUpdate events.
    /// Used as the filename for JSON state persistence.
    pub session_name: String,
    /// Number of timer cycles completed (for diagnostics).
    pub timer_cycles: usize,
    /// Accumulated elapsed time in seconds (from Timer events).
    pub elapsed_secs: f64,
    /// Last diagnostic message for the sidebar footer.
    pub last_status_msg: String,
    /// Whether permissions have been granted.
    pub permissions_granted: bool,
    /// Cumulative input tokens used across all API calls.
    pub total_input_tokens: u64,
    /// Cumulative output tokens used across all API calls.
    pub total_output_tokens: u64,
    /// The currently active (focused) tab index. Updated via TabUpdate events.
    pub active_tab_index: usize,
}

impl PluginState {
    pub fn new() -> Self {
        PluginState {
            config: PluginConfig::default(),
            sidebar_visible: true,
            panes: HashMap::new(),
            scroll_offset: 0,
            rows: 0,
            cols: 0,
            summarization_queue: VecDeque::new(),
            pending_request: None,
            click_map: Vec::new(),
            session_name: String::new(),
            timer_cycles: 0,
            elapsed_secs: 0.0,
            last_status_msg: String::new(),
            permissions_granted: false,
            total_input_tokens: 0,
            total_output_tokens: 0,
            active_tab_index: 0,
        }
    }

    /// Update the pane manifest from a PaneUpdate event.
    ///
    /// Preserves `last_scrollback_hash` for panes that already exist so that
    /// change detection continues to work across manifest updates. Panes that
    /// are no longer present in the manifest are removed.
    pub fn update_panes(&mut self, pane_manifest: &PaneManifest) {
        // Build a set of all pane keys present in the new manifest.
        let mut new_panes: HashMap<(u32, bool), PaneData> = HashMap::new();

        for (tab_index, pane_infos) in &pane_manifest.panes {
            for pane_info in pane_infos {
                let key = (pane_info.id, pane_info.is_plugin);
                let name = if pane_info.title.is_empty() {
                    format!("pane-{}", pane_info.id)
                } else {
                    pane_info.title.clone()
                };

                // Preserve existing tracking data if this pane was
                // already being tracked.
                let existing = self.panes.get(&key);
                let last_scrollback_hash = existing
                    .map(|e| e.last_scrollback_hash)
                    .unwrap_or(0);
                let summary = existing.and_then(|e| e.summary.clone());
                let last_summarized_at = existing
                    .map(|e| e.last_summarized_at)
                    .unwrap_or(0.0);

                new_panes.insert(
                    key,
                    PaneData {
                        name,
                        is_plugin: pane_info.is_plugin,
                        tab_index: *tab_index,
                        last_scrollback_hash,
                        summary,
                        last_summarized_at,
                    },
                );
            }
        }

        self.panes = new_panes;
    }
}
