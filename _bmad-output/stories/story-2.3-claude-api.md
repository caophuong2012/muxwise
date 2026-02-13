# Story 2.3: Claude API Integration and Summary Generation

## Epic
Epic 2: AI-Powered Session Summaries

## User Story
As a developer,
I want the plugin to send changed pane output to Claude API and receive a concise session summary with status color,
So that each pane gets an AI-generated context story.

## Requirements
- FR4: System sends buffered pane output to Claude API for summarization
- FR5: System generates a concise session story per pane (what it's about, where you left off, what's pending)
- FR8: System caches the last successful summary per pane

## Tasks
- [ ] Create `default-plugins/session-intelligence/src/summarize.rs` module
- [ ] Define SYSTEM_PROMPT constant that instructs AI to: summarize what the session is about, where the user left off, what needs attention. Respond with STATUS: GREEN/YELLOW/RED on the first line, then 2-3 lines of summary
- [ ] Implement build_request(scrollback: &str, config: &PluginConfig) -> (String, String, Vec<(String, String)>, BTreeMap<String, String>) returning (url, body, headers, context)
- [ ] URL: https://api.anthropic.com/v1/messages
- [ ] Headers: x-api-key from config, anthropic-version: 2023-06-01, content-type: application/json
- [ ] Body: JSON with model claude-haiku-4-5-20251001, system prompt, user message with scrollback text, max_tokens 256
- [ ] Context: BTreeMap with pane_id for request correlation
- [ ] In Timer handler, after capture: dequeue next pane from summarization_queue, call build_request, call web_request()
- [ ] Set pending_request = Some(pane_id) to block further requests until response
- [ ] Implement parse_response(body: &str) -> Option<(String, PaneStatus)> that extracts status color keyword and summary text
- [ ] Add PaneSummary struct to state.rs: text (String), status (PaneStatus enum: Active/Waiting/Error), generated_at (String timestamp), is_stale (bool)
- [ ] In update(), handle Event::WebRequestResult: parse response, update PaneData.summary, clear pending_request, process next in queue
- [ ] Never log the API key via eprintln! or display it in the sidebar

## Acceptance Criteria
- Request sent via web_request to Anthropic Messages API
- Request includes x-api-key and anthropic-version headers
- Model is set to claude-haiku-4-5-20251001
- System prompt asks for: what it's about, where left off, what's pending, plus STATUS: GREEN/YELLOW/RED
- Response parsed to extract summary text and status color
- Last successful summary cached per pane in PaneData.summary
- Sequential queue processes one pane at a time
- pending_request blocks new requests until current response arrives
- API key never logged or displayed

## Technical Context
- web_request(url, verb, headers, body, context) is non-blocking, returns WebRequestResult event
- Context BTreeMap passed back in WebRequestResult for pane correlation
- Architecture doc: _bmad-output/planning-artifacts/architecture.md
- NFR6: API key security -- never log, never display
- NFR10: Anthropic Messages API format
