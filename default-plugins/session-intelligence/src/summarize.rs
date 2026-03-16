use std::collections::BTreeMap;

use serde_json::json;
use zellij_tile::prelude::*;

use crate::sanitize;
use crate::state::{AiProvider, PaneStatus, PluginConfig};

/// System prompt that instructs the AI how to produce pane summaries.
const SYSTEM_PROMPT: &str = "\
You summarize terminal pane scrollback for a sidebar widget (28 chars wide).

Reply in EXACTLY this format — nothing else:

STATUS: GREEN|YELLOW|RED
TITLE: <short activity label, max 5 words>
<1-2 sentence summary>

STATUS rules:
- GREEN: process running normally, build/test succeeding, active work.
- YELLOW: idle shell prompt, waiting for input, paused, or completed successfully.
- RED: errors, failures, crashes, non-zero exit codes, test failures.

TITLE examples: \"cargo build\", \"npm test\", \"git rebase\", \"vim editing\", \"ssh session\", \"idle shell\".

Summary guidelines:
- Lead with the CURRENT state, not history.
- If there are errors: name the error and file/line if visible.
- If a command just finished: state the outcome (pass/fail, duration if shown).
- If idle at a prompt: say what the last meaningful command was.
- Keep it under 80 characters total. Every word must earn its place.
- Do NOT repeat the pane name or working directory — the user already sees those.";

/// Anthropic Messages API endpoint.
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// OpenAI Chat Completions API endpoint.
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenRouter Chat Completions API endpoint.
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Anthropic model to use for summarization.
const ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";

/// OpenAI model to use for summarization.
const OPENAI_MODEL: &str = "gpt-4o-mini";

/// OpenRouter model to use for summarization (cheapest capable model).
const OPENROUTER_MODEL: &str = "google/gemini-2.0-flash-lite-001";

/// Context about the pane, used to enrich the AI prompt.
pub struct PaneContext<'a> {
    pub name: &'a str,
    pub cwd: Option<&'a str>,
}

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
    pane_ctx: Option<&PaneContext>,
) -> Option<(
    String,
    HttpVerb,
    BTreeMap<String, String>,
    Vec<u8>,
    BTreeMap<String, String>,
)> {
    let api_key = config.api_key.as_ref()?;

    // Sanitize scrollback to remove sensitive data before sending to external API.
    let sanitized = sanitize::sanitize_scrollback(scrollback);

    // Focus the scrollback on the most relevant parts for summarization.
    let focused = focus_scrollback(&sanitized);

    // Build the user message with pane context.
    let user_message = build_user_message(&focused, pane_ctx);

    // Build context for request correlation.
    let mut context = BTreeMap::new();
    context.insert("pane_id".to_string(), pane_id.to_string());
    context.insert("is_plugin".to_string(), is_plugin.to_string());

    match config.ai_provider {
        AiProvider::Anthropic => build_anthropic_request(api_key, &user_message, context),
        AiProvider::OpenAi => build_openai_request(api_key, &user_message, context),
        AiProvider::OpenRouter => build_openrouter_request(api_key, &user_message, context),
    }
}

/// Build the user message with optional pane context.
fn build_user_message(scrollback: &str, pane_ctx: Option<&PaneContext>) -> String {
    let mut msg = String::new();

    if let Some(ctx) = pane_ctx {
        msg.push_str("Pane: ");
        msg.push_str(ctx.name);
        if let Some(cwd) = ctx.cwd {
            msg.push_str(" | CWD: ");
            msg.push_str(cwd);
        }
        msg.push('\n');
    }

    msg.push_str("Terminal scrollback:\n\n");
    msg.push_str(scrollback);
    msg
}

/// Focus scrollback on the most relevant parts for summarization.
///
/// Strategy:
/// 1. Always include the last 50 lines (most recent context).
/// 2. Scan for error/warning lines and include surrounding context.
/// 3. Include lines with shell prompts (command boundaries).
/// 4. Cap total output to reduce token usage.
fn focus_scrollback(scrollback: &str) -> String {
    let lines: Vec<&str> = scrollback.lines().collect();
    let total = lines.len();

    // For short scrollback, return as-is.
    if total <= 80 {
        return scrollback.to_string();
    }

    // Always take the last 50 lines.
    let tail_start = total.saturating_sub(50);
    let tail_lines = &lines[tail_start..];

    // Scan the earlier portion for important lines (errors, warnings, commands).
    let head_portion = &lines[..tail_start];
    let mut important_lines: Vec<String> = Vec::new();
    let mut important_count = 0usize;
    const MAX_IMPORTANT: usize = 30;

    for (i, line) in head_portion.iter().enumerate() {
        if important_count >= MAX_IMPORTANT {
            break;
        }
        if is_important_line(line) {
            // Include 1 line of context before and after.
            let ctx_start = i.saturating_sub(1);
            let ctx_end = (i + 2).min(head_portion.len());
            for ctx_line in &head_portion[ctx_start..ctx_end] {
                important_lines.push(ctx_line.to_string());
                important_count += 1;
            }
            important_lines.push("...".to_string());
        }
    }

    if important_lines.is_empty() {
        tail_lines.join("\n")
    } else {
        let mut result = important_lines.join("\n");
        result.push_str("\n...\n");
        result.push_str(&tail_lines.join("\n"));
        result
    }
}

/// Check if a line is likely important for summarization.
fn is_important_line(line: &str) -> bool {
    let lower = line.to_lowercase();

    // Error/failure indicators
    if lower.contains("error") || lower.contains("failed") || lower.contains("failure")
        || lower.contains("fatal") || lower.contains("panic")
        || lower.contains("exception") || lower.contains("traceback")
    {
        return true;
    }

    // Warning indicators
    if lower.contains("warning:") || lower.contains("warn:") {
        return true;
    }

    // Test results
    if lower.contains("test result") || lower.contains("tests passed")
        || lower.contains("tests failed") || lower.contains(" passed,")
    {
        return true;
    }

    // Exit codes
    if lower.contains("exit code") || lower.contains("exit status") {
        return true;
    }

    false
}

fn build_anthropic_request(
    api_key: &str,
    user_message: &str,
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
        "max_tokens": 150,
        "system": SYSTEM_PROMPT,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ]
    });

    let body = serde_json::to_vec(&body_json).unwrap_or_default();
    Some((ANTHROPIC_API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

fn build_openai_request(
    api_key: &str,
    user_message: &str,
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
        "max_tokens": 150,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": user_message
            }
        ]
    });

    let body = serde_json::to_vec(&body_json).unwrap_or_default();
    Some((OPENAI_API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

fn build_openrouter_request(
    api_key: &str,
    user_message: &str,
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
        "model": OPENROUTER_MODEL,
        "max_tokens": 150,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": user_message
            }
        ]
    });

    let body = serde_json::to_vec(&body_json).unwrap_or_default();
    Some((OPENROUTER_API_URL.to_string(), HttpVerb::Post, headers, body, context))
}

/// Token usage from an API response.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Parsed API response with summary, status, title, and token usage.
pub struct ParsedResponse {
    pub text: String,
    pub status: PaneStatus,
    pub title: Option<String>,
    pub usage: TokenUsage,
}

/// Parse the AI provider response body.
///
/// Supports both Anthropic and OpenAI response formats.
pub fn parse_response(body: &str) -> Option<ParsedResponse> {
    let response: serde_json::Value = serde_json::from_str(body).ok()?;

    let usage = parse_usage(&response);

    // Try Anthropic format: { "content": [{ "text": "..." }] }
    if let Some(text) = response
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| block.get("text"))
        .and_then(|t| t.as_str())
    {
        return parse_status_and_summary(text).map(|parsed| ParsedResponse {
            text: parsed.text,
            status: parsed.status,
            title: parsed.title,
            usage: usage.clone(),
        });
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
        return parse_status_and_summary(text).map(|parsed| ParsedResponse {
            text: parsed.text,
            status: parsed.status,
            title: parsed.title,
            usage: usage.clone(),
        });
    }

    None
}

/// Parse token usage from the API response JSON.
///
/// Anthropic: { "usage": { "input_tokens": N, "output_tokens": N } }
/// OpenAI:    { "usage": { "prompt_tokens": N, "completion_tokens": N } }
fn parse_usage(response: &serde_json::Value) -> TokenUsage {
    let usage = match response.get("usage") {
        Some(u) => u,
        None => return TokenUsage::default(),
    };

    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("prompt_tokens").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("completion_tokens").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    TokenUsage {
        input_tokens: input,
        output_tokens: output,
    }
}

/// Parsed summary result with optional title.
pub struct ParsedSummary {
    pub status: PaneStatus,
    pub title: Option<String>,
    pub text: String,
}

/// Parse a response text that starts with "STATUS: GREEN|YELLOW|RED" followed by summary lines.
fn parse_status_and_summary(text: &str) -> Option<ParsedSummary> {
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
        return Some(ParsedSummary {
            status: PaneStatus::Waiting,
            title: None,
            text: text.to_string(),
        });
    };

    // Check for optional TITLE line.
    let mut remaining_lines: Vec<&str> = lines.collect();
    let title = if let Some(first) = remaining_lines.first() {
        if let Some(title_str) = first.strip_prefix("TITLE:") {
            let t = title_str.trim().to_string();
            remaining_lines.remove(0);
            if t.is_empty() { None } else { Some(t) }
        } else {
            None
        }
    } else {
        None
    };

    let summary: String = remaining_lines
        .join("\n")
        .trim()
        .to_string();

    if summary.is_empty() {
        None
    } else {
        Some(ParsedSummary { status, title, text: summary })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_green() {
        let text = "STATUS: GREEN\nRunning cargo build.\nAll tests passing.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Active);
        assert_eq!(parsed.text, "Running cargo build.\nAll tests passing.");
        assert!(parsed.title.is_none());
    }

    #[test]
    fn parse_status_with_title() {
        let text = "STATUS: GREEN\nTITLE: cargo build\nBuilding project, 42 tests passing.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Active);
        assert_eq!(parsed.title.as_deref(), Some("cargo build"));
        assert_eq!(parsed.text, "Building project, 42 tests passing.");
    }

    #[test]
    fn parse_status_yellow() {
        let text = "STATUS: YELLOW\nTITLE: idle shell\nWaiting for user input.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Waiting);
        assert_eq!(parsed.title.as_deref(), Some("idle shell"));
        assert_eq!(parsed.text, "Waiting for user input.");
    }

    #[test]
    fn parse_status_red() {
        let text = "STATUS: RED\nTITLE: npm test\nBuild failed with 3 errors in src/main.rs.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Error);
        assert_eq!(parsed.title.as_deref(), Some("npm test"));
        assert!(parsed.text.contains("Build failed"));
    }

    #[test]
    fn parse_no_status_line() {
        let text = "This is just a summary without a status line.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Waiting);
        assert_eq!(parsed.text, "This is just a summary without a status line.");
    }

    #[test]
    fn parse_status_only_no_summary() {
        let text = "STATUS: GREEN";
        let result = parse_status_and_summary(text);
        assert!(result.is_none());
    }

    #[test]
    fn parse_status_no_title() {
        let text = "STATUS: GREEN\nAll good, nothing to see here.";
        let parsed = parse_status_and_summary(text).unwrap();
        assert_eq!(parsed.status, PaneStatus::Active);
        assert!(parsed.title.is_none());
        assert_eq!(parsed.text, "All good, nothing to see here.");
    }

    #[test]
    fn parse_response_anthropic_format() {
        let body = r#"{
            "content": [
                {
                    "type": "text",
                    "text": "STATUS: GREEN\nTITLE: cargo build\nBuilding project. All 42 tests passing."
                }
            ],
            "model": "claude-haiku-4-5-20251001",
            "role": "assistant"
        }"#;
        let parsed = parse_response(body).unwrap();
        assert_eq!(parsed.status, PaneStatus::Active);
        assert_eq!(parsed.title.as_deref(), Some("cargo build"));
        assert!(parsed.text.contains("Building project"));
    }

    #[test]
    fn parse_response_openai_format() {
        let body = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "STATUS: RED\nTITLE: npm test\nBuild failed with 2 errors in src/main.rs line 42."
                    }
                }
            ],
            "model": "gpt-4o-mini"
        }"#;
        let parsed = parse_response(body).unwrap();
        assert_eq!(parsed.status, PaneStatus::Error);
        assert_eq!(parsed.title.as_deref(), Some("npm test"));
        assert!(parsed.text.contains("Build failed"));
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
        let result = build_request(1, false, "some scrollback", &config, None);
        assert!(result.is_none());
    }

    #[test]
    fn build_request_anthropic() {
        let config = PluginConfig {
            api_key: Some("test-key-123".to_string()),
            ai_provider: AiProvider::Anthropic,
            ..PluginConfig::default()
        };
        let result = build_request(42, false, "$ cargo build", &config, None);
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
        let result = build_request(42, false, "$ npm test", &config, None);
        let (url, _, headers, body, _) = result.unwrap();
        assert_eq!(url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(
            headers.get("Authorization").unwrap(),
            "Bearer sk-test-openai-key"
        );
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body_json["model"], "gpt-4o-mini");
    }

    #[test]
    fn build_request_with_pane_context() {
        let config = PluginConfig {
            api_key: Some("test-key".to_string()),
            ai_provider: AiProvider::Anthropic,
            ..PluginConfig::default()
        };
        let ctx = PaneContext {
            name: "my-terminal",
            cwd: Some("~/project"),
        };
        let result = build_request(1, false, "$ cargo test", &config, Some(&ctx));
        let (_, _, _, body, _) = result.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let user_msg = body_json["messages"][0]["content"].as_str().unwrap();
        assert!(user_msg.contains("Pane: my-terminal"));
        assert!(user_msg.contains("CWD: ~/project"));
    }

    #[test]
    fn focus_scrollback_short() {
        let short = "line1\nline2\nline3";
        assert_eq!(focus_scrollback(short), short);
    }

    #[test]
    fn focus_scrollback_long_with_errors() {
        let mut lines: Vec<String> = (0..200).map(|i| format!("normal line {}", i)).collect();
        lines[50] = "error: compilation failed".to_string();
        let input = lines.join("\n");
        let focused = focus_scrollback(&input);
        assert!(focused.contains("error: compilation failed"));
        // Should also have the tail
        assert!(focused.contains("normal line 199"));
    }

    #[test]
    fn is_important_detects_errors() {
        assert!(is_important_line("error[E0308]: mismatched types"));
        assert!(is_important_line("FAILED: build step"));
        assert!(is_important_line("fatal: not a git repository"));
        assert!(is_important_line("warning: unused variable"));
        assert!(is_important_line("test result: 5 passed, 2 failed"));
        assert!(!is_important_line("$ cargo build"));
        assert!(!is_important_line("Compiling myproject v0.1.0"));
    }
}
