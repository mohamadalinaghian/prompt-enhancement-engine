use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use uuid::Uuid;

static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // AWS Access Key ID (standard 20 chars total: AKIA + 16 chars)
        Regex::new(r"\b(AKIA[0-9A-Z]{16,20})\b").unwrap(),
        // GitHub personal access tokens
        Regex::new(r"\b(gh[pousr]_[A-Za-z0-9_]{36,})\b").unwrap(),
        // OpenAI and standard sk- keys
        Regex::new(r"\b(sk-[a-zA-Z0-9_\-]{20,})\b").unwrap(),
        // Slack tokens
        Regex::new(r"\b(xox[baprs]-[A-Za-z0-9_\-]+)\b").unwrap(),
        // Private keys
        Regex::new(r"(-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----)").unwrap(),
        // Generic authorization bearer tokens
        Regex::new(r"(?i)\bBearer\s+([A-Za-z0-9\._\-]{25,})\b").unwrap(),
        // Connection strings with passwords
        Regex::new(r"(?i)([a-z]+://[^:\s]+:[^@\s]+@[^/\s]+)").unwrap(),
    ]
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SecretMap {
    /// Maps placeholder -> original secret
    placeholder_to_secret: HashMap<String, String>,
}

impl SecretMap {
    pub fn new() -> Self {
        Self {
            placeholder_to_secret: HashMap::new(),
        }
    }

    pub fn insert(&mut self, placeholder: String, secret: String) {
        self.placeholder_to_secret.insert(placeholder, secret);
    }

    pub fn get_secret(&self, placeholder: &str) -> Option<&str> {
        self.placeholder_to_secret
            .get(placeholder)
            .map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.placeholder_to_secret.is_empty()
    }

    pub fn len(&self) -> usize {
        self.placeholder_to_secret.len()
    }
}

pub struct SecretScanner;

impl SecretScanner {
    /// Redact detected credentials and secrets with opaque placeholders.
    pub fn redact(text: &str) -> (String, SecretMap) {
        let mut secret_map = SecretMap::new();

        let mut redacted_text = text.to_string();

        for re in SECRET_PATTERNS.iter() {
            let mut matches_to_replace = Vec::new();

            for cap in re.captures_iter(&redacted_text) {
                // If capture group 1 exists, redact only the secret token; otherwise redact whole match
                let matched_secret = if let Some(g1) = cap.get(1) {
                    g1.as_str().to_string()
                } else {
                    cap[0].to_string()
                };

                // Avoid re-redacting existing placeholders
                if !matched_secret.starts_with("[[SECRET_PE_") {
                    matches_to_replace.push(matched_secret);
                }
            }

            for secret in matches_to_replace {
                let placeholder = format!("[[SECRET_PE_{}]]", Uuid::new_v4().simple());
                redacted_text = redacted_text.replace(&secret, &placeholder);
                secret_map.insert(placeholder, secret);
            }
        }

        (redacted_text, secret_map)
    }

    /// Restore placeholders to their original values in the rendered text.
    pub fn restore(text: &str, map: &SecretMap) -> String {
        let mut restored = text.to_string();
        for (placeholder, secret) in &map.placeholder_to_secret {
            restored = restored.replace(placeholder, secret);
        }
        restored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_scanner_redact_and_restore() {
        let original =
            "Deploy using sk-1234567890abcdef1234567890 and AWS AKIAIOSFODNN7EXAMPLE35 key.";
        let (redacted, map) = SecretScanner::redact(original);

        assert!(!redacted.contains("sk-1234567890abcdef1234567890"));
        assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE35"));
        assert!(redacted.contains("[[SECRET_PE_"));
        assert_eq!(map.len(), 2);

        let restored = SecretScanner::restore(&redacted, &map);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_private_key_redaction() {
        let pkey = "-----BEGIN PRIVATE KEY-----\nMIGEAgEAMBAGByqGSM49AgEGBSuBBAAKBG0wawIBAQQgxyz...\n-----END PRIVATE KEY-----";
        let input = format!("Here is the cert:\n{}\nUse it for mTLS.", pkey);
        let (redacted, map) = SecretScanner::redact(&input);

        assert!(!redacted.contains("MIGEAgEAMBAGByqGSM49AgEGBSuBBAAKBG0wawIBAQQgxyz"));
        assert_eq!(map.len(), 1);

        let restored = SecretScanner::restore(&redacted, &map);
        assert_eq!(restored, input);
    }
}
