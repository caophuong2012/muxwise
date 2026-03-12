/// Sanitize scrollback text to remove sensitive data before sending to AI APIs.
///
/// This module provides a best-effort filter that redacts common secret patterns
/// from terminal output. It is not a security guarantee — users should be aware
/// that scrollback content is sent to external AI services and should avoid
/// displaying secrets in terminals monitored by this plugin.

/// Placeholder replacement for redacted content.
const REDACTED: &str = "[REDACTED]";

/// Patterns that indicate a line likely contains a secret value.
/// Each entry is (pattern_name, prefix_patterns).
const SECRET_ENV_KEYS: &[&str] = &[
    "API_KEY",
    "API_SECRET",
    "APIKEY",
    "SECRET_KEY",
    "SECRET",
    "TOKEN",
    "ACCESS_TOKEN",
    "AUTH_TOKEN",
    "BEARER",
    "PASSWORD",
    "PASSWD",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "DATABASE_URL",
    "DB_PASSWORD",
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "STRIPE_SECRET",
    "SENDGRID_API_KEY",
    "SLACK_TOKEN",
    "SLACK_WEBHOOK",
    "TWILIO_AUTH_TOKEN",
    "HEROKU_API_KEY",
    "NPM_TOKEN",
    "PYPI_TOKEN",
    "DOCKER_PASSWORD",
    "SSH_KEY",
];

/// Regex-like patterns matched by simple string checks.
/// These catch common secret formats regardless of context.
struct SecretPattern {
    /// A prefix that must appear (case-insensitive check done separately).
    prefix: &'static str,
    /// Minimum length of the value after the prefix to consider it a secret.
    min_value_len: usize,
}

const SECRET_PREFIXES: &[SecretPattern] = &[
    SecretPattern { prefix: "sk-", min_value_len: 20 },        // OpenAI, Stripe, Anthropic keys
    SecretPattern { prefix: "sk-ant-", min_value_len: 20 },    // Anthropic keys
    SecretPattern { prefix: "pk-", min_value_len: 20 },        // Public keys (Stripe etc)
    SecretPattern { prefix: "ghp_", min_value_len: 20 },       // GitHub personal access tokens
    SecretPattern { prefix: "gho_", min_value_len: 20 },       // GitHub OAuth tokens
    SecretPattern { prefix: "ghu_", min_value_len: 20 },       // GitHub user tokens
    SecretPattern { prefix: "ghs_", min_value_len: 20 },       // GitHub server tokens
    SecretPattern { prefix: "ghr_", min_value_len: 20 },       // GitHub refresh tokens
    SecretPattern { prefix: "xoxb-", min_value_len: 20 },      // Slack bot tokens
    SecretPattern { prefix: "xoxp-", min_value_len: 20 },      // Slack user tokens
    SecretPattern { prefix: "AKIA", min_value_len: 16 },       // AWS access key IDs
    SecretPattern { prefix: "eyJ", min_value_len: 30 },        // JWT tokens (base64 JSON)
    SecretPattern { prefix: "npm_", min_value_len: 20 },       // npm tokens
    SecretPattern { prefix: "pypi-", min_value_len: 20 },      // PyPI tokens
];

/// Sanitize scrollback text by redacting sensitive patterns.
///
/// This is applied before scrollback is sent to the AI API.
/// It performs three passes:
/// 1. Redact lines that look like env var assignments with secret keys
/// 2. Redact inline tokens that match known secret prefixes
/// 3. Redact PEM-encoded private key blocks
pub fn sanitize_scrollback(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_pem_block = false;

    for line in text.lines() {
        if in_pem_block {
            result.push_str(REDACTED);
            result.push('\n');
            if line.contains("-----END") && line.contains("PRIVATE KEY-----") {
                in_pem_block = false;
            }
            continue;
        }

        // Check for PEM private key block start.
        if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
            result.push_str(REDACTED);
            result.push('\n');
            in_pem_block = true;
            continue;
        }

        // Check for env var assignment lines with sensitive key names.
        if is_secret_assignment(line) {
            // Keep the key name but redact the value.
            if let Some(eq_pos) = line.find('=') {
                let key_part = &line[..eq_pos + 1];
                result.push_str(key_part);
                result.push_str(REDACTED);
                result.push('\n');
            } else {
                result.push_str(REDACTED);
                result.push('\n');
            }
            continue;
        }

        // Redact inline secret tokens.
        let sanitized_line = redact_inline_secrets(line);
        result.push_str(&sanitized_line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one.
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Check if a line looks like an environment variable assignment with a secret key.
///
/// Matches patterns like:
/// - `export SECRET_KEY=value`
/// - `SECRET_KEY=value`
/// - `SECRET_KEY="value"`
/// - `  SECRET_KEY = value`
/// - `.env` style: `SECRET_KEY=value`
fn is_secret_assignment(line: &str) -> bool {
    let trimmed = line.trim();

    // Strip leading "export " if present.
    let assignment = if let Some(rest) = trimmed.strip_prefix("export ") {
        rest.trim()
    } else {
        trimmed
    };

    // Must contain '=' to be an assignment.
    let eq_pos = match assignment.find('=') {
        Some(pos) => pos,
        None => return false,
    };

    let key = assignment[..eq_pos].trim().to_uppercase();

    // Check if the key name contains any of our secret key patterns.
    SECRET_ENV_KEYS.iter().any(|secret_key| key.contains(secret_key))
}

/// Redact inline tokens that match known secret prefixes.
///
/// Scans each word in the line and replaces tokens that start with
/// known secret prefixes (e.g., `sk-ant-...`, `ghp_...`, `AKIA...`).
fn redact_inline_secrets(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut last_end = 0;

    // Split on whitespace boundaries while preserving positions.
    for (start, word) in word_boundaries(line) {
        // Add any text between the last word and this one.
        result.push_str(&line[last_end..start]);

        // Strip surrounding quotes for matching.
        let stripped = word
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches(',')
            .trim_matches(';');

        if should_redact_token(stripped) {
            result.push_str(REDACTED);
        } else {
            result.push_str(word);
        }

        last_end = start + word.len();
    }

    // Add any trailing text.
    result.push_str(&line[last_end..]);
    result
}

/// Check if a token matches known secret prefixes and is long enough.
fn should_redact_token(token: &str) -> bool {
    for pattern in SECRET_PREFIXES {
        if token.starts_with(pattern.prefix) && token.len() >= pattern.prefix.len() + pattern.min_value_len {
            return true;
        }
    }
    false
}

/// Iterate over word boundaries in a string, yielding (start_index, word_str).
fn word_boundaries(s: &str) -> Vec<(usize, &str)> {
    let mut words = Vec::new();
    let mut in_word = false;
    let mut word_start = 0;

    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            if in_word {
                words.push((word_start, &s[word_start..i]));
                in_word = false;
            }
        } else if !in_word {
            word_start = i;
            in_word = true;
        }
    }

    if in_word {
        words.push((word_start, &s[word_start..]));
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_export_api_key() {
        let input = "export API_KEY=sk-ant-abc123def456ghi789";
        let result = sanitize_scrollback(input);
        assert!(result.contains("API_KEY="));
        assert!(result.contains(REDACTED));
        assert!(!result.contains("sk-ant-abc123def456ghi789"));
    }

    #[test]
    fn redacts_env_assignment() {
        let input = "OPENAI_API_KEY=sk-1234567890abcdef1234567890abcdef";
        let result = sanitize_scrollback(input);
        assert!(result.contains("OPENAI_API_KEY="));
        assert!(result.contains(REDACTED));
    }

    #[test]
    fn redacts_quoted_assignment() {
        let input = r#"export SECRET_KEY="my-super-secret-value""#;
        let result = sanitize_scrollback(input);
        assert!(result.contains("SECRET_KEY="));
        assert!(!result.contains("my-super-secret-value"));
    }

    #[test]
    fn redacts_aws_access_key() {
        let input = "Your key is AKIAIOSFODNN7EXAMPLE";
        let result = sanitize_scrollback(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn redacts_github_token() {
        let input = "Using token ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef";
        let result = sanitize_scrollback(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    }

    #[test]
    fn redacts_slack_token() {
        // Use a clearly fake token pattern to avoid GitHub push protection
        let input = "SLACK_TOKEN=xoxb-fake-test-only-not-real-token";
        let result = sanitize_scrollback(input);
        assert!(result.contains("SLACK_TOKEN="));
        assert!(result.contains(REDACTED));
        assert!(!result.contains("fake-test-only"));
    }

    #[test]
    fn redacts_pem_private_key() {
        let input = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA0Z3VS5JJcds3xfn\n-----END RSA PRIVATE KEY-----";
        let result = sanitize_scrollback(input);
        assert!(!result.contains("MIIEpAIBAAKCAQEA0Z3VS5JJcds3xfn"));
        assert_eq!(result.matches(REDACTED).count(), 3); // begin, content, end
    }

    #[test]
    fn preserves_normal_lines() {
        let input = "$ cargo build\n   Compiling myproject v0.1.0\n    Finished release target";
        let result = sanitize_scrollback(input);
        assert_eq!(result, input);
    }

    #[test]
    fn preserves_short_sk_prefix() {
        // "sk-1" is too short to be a real key
        let input = "variable sk-1 is set";
        let result = sanitize_scrollback(input);
        assert_eq!(result, input);
    }

    #[test]
    fn redacts_jwt_token() {
        let input = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkw";
        let result = sanitize_scrollback(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }

    #[test]
    fn redacts_database_url() {
        let input = "DATABASE_URL=postgres://user:password@host:5432/db";
        let result = sanitize_scrollback(input);
        assert!(result.contains("DATABASE_URL="));
        assert!(result.contains(REDACTED));
        assert!(!result.contains("password"));
    }

    #[test]
    fn redacts_inline_anthropic_key() {
        let input = "curl -H 'x-api-key: sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234567890'";
        let result = sanitize_scrollback(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("sk-ant-api03"));
    }

    #[test]
    fn handles_multiple_secrets_on_one_line() {
        let input = "Keys: AKIAIOSFODNN7EXAMPLE and ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef";
        let result = sanitize_scrollback(input);
        assert_eq!(result.matches(REDACTED).count(), 2);
    }
}
