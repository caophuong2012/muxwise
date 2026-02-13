use std::collections::BTreeMap;

use serde_json::json;
use zellij_tile::prelude::*;

use crate::state::{PaneStatus, PluginConfig};

/// System prompt that instructs the AI how to produce pane summaries.
const SYSTEM_PROMPT: &str = "\
You are a terminal session summarizer. Given the scrollback output of a terminal pane, \
produce a concise summary. Your response MUST follow this exact format:

STATUS: GREEN|YELLOW|RED
<2-3 line summary>

STATUS meanings:
- GREEN: The session is actively working, processes running normally, no errors.
- YELLOW: The session is idle, waiting for user input, or paused.
- RED: There are errors, failures, or problems that need attention.

Your summary should cover:
1. What this session is about (e.g., running tests, editing code, building a project).
2. Where the user left off (e.g., last command run, current state).
3. What needs attention (e.g., errors to fix, pending actions).

Keep the summary to 2-3 lines maximum. Be concise and actionable.";

/// Anthropic Messages API endpoint.
const API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Model to use for summarization.
const MODEL: &str = "claude-haiku-4-5-20251001";

/// Build an HTTP request for the Anthropic Messages API.
///
/// Returns `(url, verb, headers, body, context)` suitable for passing to `web_request()`.
///
/// Returns `None` if the API key is not configured.
pub fn build_request(
    pane_id: u32,
    is_plugin: bool,
    scrollback: &str,
    config: &PluginConfig,
) -> Option<(
    String,
    HttpVerb,
    BTreeMap<String, String>,
    Vec<u8>,
    BTreeMap<String, String>,
)> {
    let api_key = config.api_key.as_ref()?;

    // Build headers.
    let mut headers = BTreeMap::new();
    headers.insert("x-api-key".to_string(), api_key.clone());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());

    // Build JSON body per Anthropic Messages API format.
    let body_json = json!({
        "model": MODEL,
        "max_tokens": 256,
        "system": SYSTEM_PROMPT,
        "messages": [
            {
                "role": "user",
                "content": format!(
                    "Here is the terminal scrollback output for this pane:\n\n{}",
                    scrollback
                )
            }
        ]
    });

    let body = serde_json::to_vec(&body_json).unwrap_or_default();

    // Build context for request correlation.
    let mut context = BTreeMap::new();
    context.insert("pane_id".to_string(), pane_id.to_string());
    context.insert("is_plugin".to_string(), is_plugin.to_string());

    Some((API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

/// Parse the Anthropic Messages API response body.
///
/// Extracts the text content from the response, then parses the STATUS: line
/// and the summary text that follows it.
///
/// Returns `None` if the response cannot be parsed.
pub fn parse_response(body: &str) -> Option<(String, PaneStatus)> {
    // Parse the JSON response.
    let response: serde_json::Value = serde_json::from_str(body).ok()?;

    // Extract the text from the first content block.
    let content = response.get("content")?.as_array()?;
    let first_block = content.first()?;
    let text = first_block.get("text")?.as_str()?;

    // Parse the STATUS: line and summary.
    parse_status_and_summary(text)
}

/// Parse a response text that starts with "STATUS: GREEN|YELLOW|RED" followed by summary lines.
fn parse_status_and_summary(text: &str) -> Option<(String, PaneStatus)> {
    let text = text.trim();
    let mut lines = text.lines();

    // First line should be "STATUS: <color>"
    let status_line = lines.next()?;
    let status = if let Some(color) = status_line.strip_prefix("STATUS:") {
        match color.trim().to_uppercase().as_str() {
            "GREEN" => PaneStatus::Active,
            "YELLOW" => PaneStatus::Waiting,
            "RED" => PaneStatus::Error,
            _ => PaneStatus::Waiting,
        }
    } else {
        // If no STATUS: line, treat entire text as summary with default status.
        return Some((text.to_string(), PaneStatus::Waiting));
    };

    // Remaining lines are the summary text.
    let summary: String = lines
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string();

    if summary.is_empty() {
        // If we got a status but no summary text, return None.
        None
    } else {
        Some((summary, status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_green() {
        let text = "STATUS: GREEN\nRunning cargo build.\nAll tests passing.";
        let (summary, status) = parse_status_and_summary(text).unwrap();
        assert_eq!(status, PaneStatus::Active);
        assert_eq!(summary, "Running cargo build.\nAll tests passing.");
    }

    #[test]
    fn parse_status_yellow() {
        let text = "STATUS: YELLOW\nWaiting for user input at the shell prompt.";
        let (summary, status) = parse_status_and_summary(text).unwrap();
        assert_eq!(status, PaneStatus::Waiting);
        assert_eq!(summary, "Waiting for user input at the shell prompt.");
    }

    #[test]
    fn parse_status_red() {
        let text = "STATUS: RED\nCompilation failed with 3 errors.\nSee output above.";
        let (summary, status) = parse_status_and_summary(text).unwrap();
        assert_eq!(status, PaneStatus::Error);
        assert_eq!(summary, "Compilation failed with 3 errors.\nSee output above.");
    }

    #[test]
    fn parse_no_status_line() {
        let text = "This is just a summary without a status line.";
        let (summary, status) = parse_status_and_summary(text).unwrap();
        assert_eq!(status, PaneStatus::Waiting);
        assert_eq!(summary, "This is just a summary without a status line.");
    }

    #[test]
    fn parse_status_only_no_summary() {
        let text = "STATUS: GREEN";
        let result = parse_status_and_summary(text);
        assert!(result.is_none());
    }

    #[test]
    fn parse_response_valid_json() {
        let body = r#"{
            "content": [
                {
                    "type": "text",
                    "text": "STATUS: GREEN\nBuilding project with cargo.\nAll 42 tests passing."
                }
            ],
            "model": "claude-haiku-4-5-20251001",
            "role": "assistant"
        }"#;
        let (summary, status) = parse_response(body).unwrap();
        assert_eq!(status, PaneStatus::Active);
        assert!(summary.contains("Building project"));
    }

    #[test]
    fn parse_response_invalid_json() {
        let body = "not valid json";
        assert!(parse_response(body).is_none());
    }

    #[test]
    fn parse_response_missing_content() {
        let body = r#"{"error": "something went wrong"}"#;
        assert!(parse_response(body).is_none());
    }

    #[test]
    fn build_request_no_api_key() {
        let config = PluginConfig::default();
        assert!(config.api_key.is_none());
        let result = build_request(1, false, "some scrollback", &config);
        assert!(result.is_none());
    }

    #[test]
    fn build_request_with_api_key() {
        let config = PluginConfig {
            api_key: Some("test-key-123".to_string()),
            ..PluginConfig::default()
        };
        let result = build_request(42, false, "$ cargo build\nCompiling...", &config);
        assert!(result.is_some());

        let (url, verb, headers, body, context) = result.unwrap();
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
        assert!(matches!(verb, HttpVerb::Post));
        assert_eq!(headers.get("x-api-key").unwrap(), "test-key-123");
        assert_eq!(headers.get("anthropic-version").unwrap(), "2023-06-01");
        assert_eq!(headers.get("content-type").unwrap(), "application/json");

        // Verify body is valid JSON with expected fields.
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["model"], "claude-haiku-4-5-20251001");
        assert_eq!(body_json["max_tokens"], 256);
        assert!(body_json["system"].as_str().unwrap().contains("terminal session summarizer"));

        // Verify context for request correlation.
        assert_eq!(context.get("pane_id").unwrap(), "42");
        assert_eq!(context.get("is_plugin").unwrap(), "false");
    }
}
