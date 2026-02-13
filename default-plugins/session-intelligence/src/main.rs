mod capture;
mod persistence;
mod sidebar;
mod state;
mod summarize;

use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use sidebar::render_sidebar;
use state::{PaneSummary, PluginConfig, PluginState};

register_plugin!(PluginState);

impl PluginState {
    /// Dequeue the next pane from the summarization queue and send an API request.
    ///
    /// Does nothing if a request is already pending, the queue is empty,
    /// or no API key is configured.
    fn dequeue_next_summarization(&mut self) {
        // Don't send a new request if one is already in flight.
        if self.pending_request.is_some() {
            return;
        }

        // Don't send requests if no API key is configured.
        if self.config.api_key.is_none() {
            return;
        }

        // Dequeue the next pane.
        let (pane_id, is_plugin) = match self.summarization_queue.pop_front() {
            Some(entry) => entry,
            None => return,
        };

        // Fetch the scrollback for this pane.
        let scrollback =
            capture::fetch_scrollback(pane_id, is_plugin, self.config.buffer_size_lines);

        if scrollback.is_empty() {
            eprintln!(
                "session-intelligence: skipping pane {} (empty scrollback)",
                pane_id
            );
            // Try the next one in the queue.
            self.dequeue_next_summarization();
            return;
        }

        // Build and send the API request.
        if let Some((url, verb, headers, body, context)) =
            summarize::build_request(pane_id, is_plugin, &scrollback, &self.config)
        {
            eprintln!(
                "session-intelligence: sending summarization request for pane {}",
                pane_id
            );
            web_request(&url, verb, headers, body, context);
            self.pending_request = Some((pane_id, is_plugin));
        }
    }
}

impl ZellijPlugin for PluginState {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        // Request all permissions needed by this plugin (current + future stories).
        request_permission(&[
            PermissionType::ReadPaneContents,
            PermissionType::WebAccess,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
            PermissionType::FullHdAccess,
        ]);

        // Subscribe to the events this plugin will handle.
        subscribe(&[
            EventType::Timer,
            EventType::Key,
            EventType::Mouse,
            EventType::PaneUpdate,
            EventType::SessionUpdate,
            EventType::WebRequestResult,
            EventType::PermissionRequestResult,
        ]);

        // Parse configuration from KDL config values.
        self.config = PluginConfig::from_btree(&configuration);

        // Initialize default state.
        self.sidebar_visible = true;

        // Try to discover session name from configuration, falling back to "default".
        self.session_name = configuration
            .get("session_name")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());

        // Attempt to restore persisted state from disk.
        if let Some(persisted) = persistence::load_state(&self.session_name) {
            persistence::restore_into(&persisted, self);
            eprintln!(
                "session-intelligence: restored persisted state for session '{}'",
                self.session_name
            );
        }

        // Start the periodic summarization timer chain.
        set_timeout(self.config.summarization_interval_secs);
        eprintln!(
            "session-intelligence plugin loaded (summarization interval: {}s, session: '{}')",
            self.config.summarization_interval_secs, self.session_name
        );
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::PaneUpdate(pane_manifest) => {
                self.update_panes(&pane_manifest);
                should_render = true;
            },
            Event::Timer(_elapsed) => {
                // A summarization cycle has been triggered by the timer.
                eprintln!("session-intelligence: summarization cycle triggered");

                // Snapshot the pane keys to avoid borrowing issues.
                let pane_keys: Vec<(u32, bool)> = self.panes.keys().cloned().collect();

                for (pane_id, is_plugin) in pane_keys {
                    // Skip plugin panes entirely -- we don't summarize plugins
                    // (this also filters out our own sidebar pane).
                    if is_plugin {
                        continue;
                    }

                    let buffer_size = self.config.buffer_size_lines;
                    let scrollback = capture::fetch_scrollback(pane_id, is_plugin, buffer_size);
                    let new_hash = capture::hash_scrollback(&scrollback);

                    if let Some(pane_data) = self.panes.get_mut(&(pane_id, is_plugin)) {
                        if new_hash != pane_data.last_scrollback_hash {
                            eprintln!(
                                "session-intelligence: pane {} scrollback changed (hash {} -> {})",
                                pane_id, pane_data.last_scrollback_hash, new_hash
                            );
                            pane_data.last_scrollback_hash = new_hash;

                            // Only enqueue if this pane is not already in the queue.
                            if !self.summarization_queue.contains(&(pane_id, is_plugin)) {
                                self.summarization_queue.push_back((pane_id, is_plugin));
                            }
                        }
                    }
                }

                if !self.summarization_queue.is_empty() {
                    eprintln!(
                        "session-intelligence: {} pane(s) queued for summarization",
                        self.summarization_queue.len()
                    );
                }

                // Dequeue and send the next summarization request if none is pending.
                self.dequeue_next_summarization();

                // Re-arm the timer to maintain the periodic schedule.
                set_timeout(self.config.summarization_interval_secs);

                should_render = false;
            },
            Event::Key(key) => {
                if key.bare_key == BareKey::Tab && key.has_no_modifiers() {
                    self.sidebar_visible = !self.sidebar_visible;
                    should_render = true;

                    // Persist sidebar visibility change.
                    let session_name = self.session_name.clone();
                    persistence::save_state(self, &session_name);
                }
            },
            Event::Mouse(mouse_event) => {
                if let Mouse::LeftClick(line, _col) = mouse_event {
                    let row = line as usize;
                    if row < self.click_map.len() {
                        if let Some((pane_id, is_plugin)) = self.click_map[row] {
                            let target = if is_plugin {
                                PaneId::Plugin(pane_id)
                            } else {
                                PaneId::Terminal(pane_id)
                            };
                            focus_pane_with_id(target, false, true);
                        }
                    }
                }
            },
            Event::SessionUpdate(session_infos, _resurrectable_sessions) => {
                // Discover the current session name from the session update.
                for session_info in &session_infos {
                    if session_info.is_current_session {
                        let new_name = session_info.name.clone();
                        if !new_name.is_empty() && new_name != self.session_name {
                            eprintln!(
                                "session-intelligence: session name discovered: '{}'",
                                new_name
                            );
                            self.session_name = new_name;

                            // Try to load persisted state for the newly discovered session.
                            if let Some(persisted) =
                                persistence::load_state(&self.session_name)
                            {
                                persistence::restore_into(&persisted, self);
                                should_render = true;
                            }
                        }
                        break;
                    }
                }

                // Story 3.2: On re-attach, queue ALL known panes for summarization
                // regardless of hash comparison so summaries refresh immediately.
                let pane_keys: Vec<(u32, bool)> = self
                    .panes
                    .keys()
                    .filter(|(_id, is_plugin)| !is_plugin)
                    .cloned()
                    .collect();
                for key in pane_keys {
                    if !self.summarization_queue.contains(&key) {
                        self.summarization_queue.push_back(key);
                    }
                }
                if !self.summarization_queue.is_empty() {
                    eprintln!(
                        "session-intelligence: re-attach detected, queued {} pane(s) for summarization",
                        self.summarization_queue.len()
                    );
                    self.dequeue_next_summarization();
                }
            },
            Event::WebRequestResult(status_code, _headers, body, context) => {
                // Clear the pending request so the next one can be sent.
                self.pending_request = None;

                if status_code == 200 {
                    // Parse the response body.
                    let body_str = String::from_utf8_lossy(&body);
                    if let Some((summary_text, pane_status)) =
                        summarize::parse_response(&body_str)
                    {
                        // Extract pane_id and is_plugin from the context.
                        let pane_id = context
                            .get("pane_id")
                            .and_then(|v| v.parse::<u32>().ok());
                        let is_plugin = context
                            .get("is_plugin")
                            .map(|v| v == "true")
                            .unwrap_or(false);

                        if let Some(pane_id) = pane_id {
                            let key = (pane_id, is_plugin);
                            let mut summary_updated = false;
                            if let Some(pane_data) = self.panes.get_mut(&key) {
                                eprintln!(
                                    "session-intelligence: summary updated for pane {} (status: {:?})",
                                    pane_id, pane_status
                                );
                                pane_data.summary = Some(PaneSummary {
                                    text: summary_text,
                                    status: pane_status,
                                    generated_at: String::new(), // Timestamp not available in WASM
                                    is_stale: false,
                                });
                                should_render = true;
                                summary_updated = true;
                            }

                            // Persist state to disk after each summary update.
                            if summary_updated {
                                let session_name = self.session_name.clone();
                                persistence::save_state(self, &session_name);
                            }
                        }
                    } else {
                        eprintln!(
                            "session-intelligence: failed to parse API response body"
                        );
                    }
                } else {
                    // Story 3.3: Graceful API failure handling.
                    // Preserve last successful summary, mark as stale.
                    let body_str = String::from_utf8_lossy(&body);
                    eprintln!(
                        "session-intelligence: API request failed with status {}: {}",
                        status_code,
                        &body_str[..body_str.len().min(200)]
                    );

                    // Extract pane info from context to mark summary as stale.
                    let pane_id = context
                        .get("pane_id")
                        .and_then(|v| v.parse::<u32>().ok());
                    let is_plugin = context
                        .get("is_plugin")
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    if let Some(pane_id) = pane_id {
                        if let Some(pane_data) = self.panes.get_mut(&(pane_id, is_plugin)) {
                            // Set is_stale on existing summary (don't clear it).
                            if let Some(ref mut summary) = pane_data.summary {
                                summary.is_stale = true;
                                should_render = true;
                            }
                        }
                    }

                    // On 429 rate limit: don't retry immediately, let the next
                    // timer cycle handle it. Clear the queue for this cycle.
                    if status_code == 429 {
                        eprintln!(
                            "session-intelligence: rate limited (429), backing off to next timer cycle"
                        );
                        self.summarization_queue.clear();
                    }
                }

                // Dequeue the next pane if the queue is not empty.
                self.dequeue_next_summarization();
            },
            Event::PermissionRequestResult(result) => {
                match result {
                    PermissionStatus::Granted => {
                        eprintln!("session-intelligence: permissions granted");
                    },
                    PermissionStatus::Denied => {
                        eprintln!("session-intelligence: permissions denied");
                    },
                }
                should_render = true;
            },
            _ => {},
        }

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        render_sidebar(self, rows, cols);
    }
}
