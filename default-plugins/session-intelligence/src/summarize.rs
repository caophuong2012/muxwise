use std::collections::BTreeMap;

use serde_json::json;
use zellij_tile::prelude::*;

use crate::state::{AiProvider, PaneStatus, PluginConfig};

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
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// OpenAI Chat Completions API endpoint.
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Anthropic model to use for summarization.
const ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";

/// OpenAI model to use for summarization.
const OPENAI_MODEL: &str = "gpt-4o-mini";

/// Build an HTTP request for the configured AI provider.
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

    // Build context for request correlation.
    let mut context = BTreeMap::new();
    context.insert("pane_id".to_string(), pane_id.to_string());
    context.insert("is_plugin".to_string(), is_plugin.to_string());

    match config.ai_provider {
        AiProvider::Anthropic => build_anthropic_request(api_key, scrollback, context),
        AiProvider::OpenAi => build_openai_request(api_key, scrollback, context),
    }
}

fn build_anthropic_request(
    api_key: &str,
    scrollback: &str,
    context: BTreeMap<String, String>,
) -> Option<(
    String,
    HttpVerb,
    BTreeMap<String, String>,
    Vec<u8>,
    BTreeMap<String, String>,
)> {
    let mut headers = BTreeMap::new();
    headers.insert("x-api-key".to_string(), api_key.to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    headers.insert("content-type".to_string(), "application/json".to_string());

    let body_json = json!({
        "model": ANTHROPIC_MODEL,
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
    Some((ANTHROPIC_API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

fn build_openai_request(
    api_key: &str,
    scrollback: &str,
    context: BTreeMap<String, String>,
) -> Option<(
    String,
    HttpVerb,
    BTreeMap<String, String>,
    Vec<u8>,
    BTreeMap<String, String>,
)> {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", api_key),
    );
    headers.insert("content-type".to_string(), "application/json".to_string());

    let body_json = json!({
        "model": OPENAI_MODEL,
        "max_tokens": 256,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
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
    Some((OPENAI_API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

/// Parse the AI provider response body.
///
/// Supports both Anthropic and OpenAI response formats.
pub fn parse_response(body: &str) -> Option<(String, PaneStatus)> {
    let response: serde_json::Value = serde_json::from_str(body).ok()?;

    // Try Anthropic format: { "content": [{ "text": "..." }] }
    if let Some(text) = response
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| block.get("text"))
        .and_then(|t| t.as_str())
    {
        return parse_status_and_summary(text);
    }

    // Try OpenAI format: { "choices": [{ "message": { "content": "..." } }] }
    if let Some(text) = response
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|t| t.as_str())
    {
        return parse_status_and_summary(text);
    }

    None
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
    fn parse_response_anthropic_format() {
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
    fn parse_response_openai_format() {
        let body = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "STATUS: RED\nBuild failed with 2 errors.\nCheck src/main.rs line 42."
                    }
                }
            ],
            "model": "gpt-4o-mini"
        }"#;
        let (summary, status) = parse_response(body).unwrap();
        assert_eq!(status, PaneStatus::Error);
        assert!(summary.contains("Build failed"));
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
    fn build_request_anthropic() {
        let config = PluginConfig {
            api_key: Some("test-key-123".to_string()),
            ai_provider: AiProvider::Anthropic,
            ..PluginConfig::default()
        };
        let result = build_request(42, false, "$ cargo build", &config);
        let (url, _, headers, body, _) = result.unwrap();
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
        assert_eq!(headers.get("x-api-key").unwrap(), "test-key-123");
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["model"], "claude-haiku-4-5-20251001");
    }

    #[test]
    fn build_request_openai() {
        let config = PluginConfig {
            api_key: Some("sk-test-openai-key".to_string()),
            ai_provider: AiProvider::OpenAi,
            ..PluginConfig::default()
        };
        let result = build_request(42, false, "$ npm test", &config);
        let (url, _, headers, body, _) = result.unwrap();
        assert_eq!(url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(
            headers.get("Authorization").unwrap(),
            "Bearer sk-test-openai-key"
        );
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["model"], "gpt-4o-mini");
    }
}
