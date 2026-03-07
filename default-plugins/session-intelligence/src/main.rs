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
    /// Run a summarization scan: check scrollback hashes and queue changed panes.
    fn run_summarization_scan(&mut self) {
        let active_tab = self.active_tab_index;
        let pane_keys: Vec<(u32, bool)> = self
            .panes
            .iter()
            .filter(|(_, data)| data.tab_index == active_tab)
            .map(|(key, _)| *key)
            .collect();
        let terminal_pane_count = pane_keys.iter().filter(|(_, p)| !p).count();

        if terminal_pane_count == 0 {
            self.last_status_msg = format!(
                "Scan #{}: no terminal panes",
                self.timer_cycles
            );
            return;
        }

        let now = self.elapsed_secs;
        let cooldown = self.config.cooldown_secs;
        let mut queued_count = 0usize;

        for (pane_id, is_plugin) in pane_keys {
            if is_plugin {
                continue;
            }

            let buffer_size = self.config.buffer_size_lines;
            let scrollback = capture::fetch_scrollback(pane_id, is_plugin, buffer_size);
            let new_hash = capture::hash_scrollback(&scrollback);

            if let Some(pane_data) = self.panes.get_mut(&(pane_id, is_plugin)) {
                if new_hash != pane_data.last_scrollback_hash {
                    // Check per-pane cooldown: skip if summarized too recently.
                    let since_last = now - pane_data.last_summarized_at;
                    if pane_data.last_summarized_at > 0.0 && since_last < cooldown {
                        eprintln!(
                            "session-intelligence: pane {} changed but cooldown active ({:.0}s/{:.0}s)",
                            pane_id, since_last, cooldown
                        );
                        // Update hash so we don't keep re-checking the same content,
                        // but mark summary as stale so user sees it needs refresh.
                        pane_data.last_scrollback_hash = new_hash;
                        if let Some(ref mut summary) = pane_data.summary {
                            summary.is_stale = true;
                        }
                        continue;
                    }

                    eprintln!(
                        "session-intelligence: pane {} scrollback changed (hash {} -> {}), queuing",
                        pane_id, pane_data.last_scrollback_hash, new_hash
                    );
                    pane_data.last_scrollback_hash = new_hash;
                    if !self.summarization_queue.contains(&(pane_id, is_plugin)) {
                        self.summarization_queue.push_back((pane_id, is_plugin));
                        queued_count += 1;
                    }
                }
            }
        }

        self.last_status_msg = format!(
            "Scan #{}: {} pane(s), {} queued",
            self.timer_cycles, terminal_pane_count, queued_count
        );

        self.dequeue_next_summarization();
    }

    /// Dequeue the next pane from the summarization queue and send an API request.
    ///
    /// Does nothing if a request is already pending, the queue is empty,
    /// or no API key is configured.
    fn dequeue_next_summarization(&mut self) {
        if self.pending_request.is_some() || self.config.api_key.is_none() {
            return;
        }

        let (pane_id, is_plugin) = match self.summarization_queue.pop_front() {
            Some(entry) => entry,
            None => return,
        };

        let scrollback =
            capture::fetch_scrollback(pane_id, is_plugin, self.config.buffer_size_lines);

        if scrollback.is_empty() {
            self.dequeue_next_summarization();
            return;
        }

        if let Some((url, verb, headers, body, context)) =
            summarize::build_request(pane_id, is_plugin, &scrollback, &self.config)
        {
            eprintln!(
                "session-intelligence: sending API request for pane {} ({} bytes)",
                pane_id, body.len()
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
            EventType::TabUpdate,
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

        // Start with a short initial timer (5s) so the first cycle fires quickly,
        // then subsequent cycles use the configured interval.
        set_timeout(5.0);
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
                self.last_status_msg = format!(
                    "Panes: {} total",
                    self.panes.len()
                );
                should_render = true;
            },
            Event::TabUpdate(tab_infos) => {
                for tab_info in &tab_infos {
                    if tab_info.active {
                        if self.active_tab_index != tab_info.position {
                            self.active_tab_index = tab_info.position;
                            should_render = true;
                        }
                        break;
                    }
                }
            },
            Event::Timer(elapsed) => {
                self.timer_cycles += 1;
                self.elapsed_secs += elapsed as f64;
                self.run_summarization_scan();
                set_timeout(self.config.summarization_interval_secs);
                should_render = true;
            },
            Event::Key(key) => {
                if key.bare_key == BareKey::Tab && key.has_no_modifiers() {
                    self.sidebar_visible = !self.sidebar_visible;
                    should_render = true;

                    // Persist sidebar visibility change.
                    let session_name = self.session_name.clone();
                    persistence::save_state(self, &session_name);
                } else if key.bare_key == BareKey::Char('s') && key.has_no_modifiers() {
                    // Manual summarization trigger: press 's' while focused on plugin pane.
                    self.timer_cycles += 1;

                    // Step 1: Test if we can read pane scrollback (active tab only).
                    let active_tab = self.active_tab_index;
                    let pane_keys: Vec<(u32, bool)> = self
                        .panes
                        .iter()
                        .filter(|((_, is_plugin), data)| !is_plugin && data.tab_index == active_tab)
                        .map(|(key, _)| *key)
                        .collect();

                    if pane_keys.is_empty() {
                        self.last_status_msg = "No terminal panes to scan".to_string();
                    } else {
                        let (pane_id, is_plugin) = pane_keys[0];
                        let scrollback = capture::fetch_scrollback(
                            pane_id,
                            is_plugin,
                            self.config.buffer_size_lines,
                        );
                        let scroll_len = scrollback.len();
                        if scrollback.is_empty() {
                            self.last_status_msg = format!(
                                "Pane {} scrollback: EMPTY",
                                pane_id
                            );
                        } else {
                            // Scrollback works! Build and send API request.
                            self.last_status_msg = format!(
                                "Pane {} scrollback: {} chars",
                                pane_id, scroll_len
                            );
                            if let Some((url, verb, headers, body, context)) =
                                summarize::build_request(
                                    pane_id,
                                    is_plugin,
                                    &scrollback,
                                    &self.config,
                                )
                            {
                                web_request(&url, verb, headers, body, context);
                                self.pending_request = Some((pane_id, is_plugin));
                                self.last_status_msg = format!(
                                    "Sent API req for pane {} ({}ch)",
                                    pane_id, scroll_len
                                );
                            }
                        }
                    }
                    should_render = true;
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
                // Note: we no longer force-queue all panes on SessionUpdate.
                // The timer-based scan with hash checks handles summarization,
                // avoiding redundant API calls when nothing has changed.
            },
            Event::WebRequestResult(status_code, _headers, body, context) => {
                self.pending_request = None;
                self.last_status_msg = format!("API response: {}", status_code);

                let pane_id = context
                    .get("pane_id")
                    .and_then(|v| v.parse::<u32>().ok());
                let is_plugin = context
                    .get("is_plugin")
                    .map(|v| v == "true")
                    .unwrap_or(false);

                if status_code == 200 {
                    let body_str = String::from_utf8_lossy(&body);
                    if let Some((summary_text, pane_status, usage)) =
                        summarize::parse_response(&body_str)
                    {
                        self.total_input_tokens += usage.input_tokens;
                        self.total_output_tokens += usage.output_tokens;

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
                                    generated_at: String::new(),
                                    is_stale: false,
                                });
                                pane_data.last_summarized_at = self.elapsed_secs;
                                should_render = true;
                                summary_updated = true;
                            }

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
                    let body_str = String::from_utf8_lossy(&body);
                    eprintln!(
                        "session-intelligence: API failed status={}: {}",
                        status_code,
                        &body_str[..body_str.len().min(200)]
                    );

                    if let Some(pane_id) = pane_id {
                        if let Some(pane_data) = self.panes.get_mut(&(pane_id, is_plugin)) {
                            if let Some(ref mut summary) = pane_data.summary {
                                summary.is_stale = true;
                                should_render = true;
                            }
                        }
                    }

                    if status_code == 429 {
                        self.summarization_queue.clear();
                    }
                }

                self.dequeue_next_summarization();
            },
            Event::PermissionRequestResult(result) => {
                match result {
                    PermissionStatus::Granted => {
                        eprintln!("session-intelligence: permissions granted");
                        self.permissions_granted = true;
                        self.last_status_msg = "Permissions granted".to_string();
                    },
                    PermissionStatus::Denied => {
                        eprintln!("session-intelligence: permissions denied");
                        self.permissions_granted = false;
                        self.last_status_msg = "Permissions DENIED".to_string();
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
