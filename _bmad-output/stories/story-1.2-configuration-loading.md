# Story 1.2: Configuration Loading from KDL

## Epic
Epic 1: Plugin Foundation & Sidebar Shell

## User Story
As a developer,
I want to configure my API key, summarization interval, and buffer size in Zellij's KDL config file,
So that the plugin reads my preferences on load with sensible defaults for optional values.

## Requirements
- FR18: User can configure their AI API key in Zellij's KDL config file
- FR19: User can optionally configure the summarization interval
- FR20: User can optionally configure the output buffer size per pane
- FR21: System uses sensible defaults for all optional configuration values

## Tasks
- [ ] Add PluginConfig struct to state.rs with fields: api_key (Option<String>), summarization_interval_secs (f64), buffer_size_lines (usize)
- [ ] Implement PluginConfig::from_btree(config: &BTreeMap<String, String>) that parses ai_api_key, summarization_interval, buffer_size
- [ ] Set default summarization_interval to 120 seconds when not provided
- [ ] Set default buffer_size to 2000 lines when not provided
- [ ] Log warning via eprintln! when API key is missing (do not crash)
- [ ] Call PluginConfig::from_btree in the load() method and store in PluginState.config
- [ ] Override defaults with user-provided values when present in config

## Acceptance Criteria
- API key read from KDL config `ai_api_key` field and stored in PluginConfig
- summarization_interval defaults to 120 seconds if not provided
- buffer_size defaults to 2000 lines if not provided
- Provided values override the defaults
- Missing API key logs a warning via eprintln! but does not crash the plugin
- PluginConfig is stored in PluginState.config

## Technical Context
- KDL config values arrive as BTreeMap<String, String> in load() method
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- NFR6: API key never logged, displayed in UI, or transmitted except to Claude API
