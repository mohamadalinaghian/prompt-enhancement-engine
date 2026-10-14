use crate::envelope::SummarizerEnvelope;
use crate::traits::ContextSummarizer;
use async_trait::async_trait;
use pe_context::{Classification, EvidenceBundle};
use pe_core::CoreError;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::time::Duration;

static SUMMARIZER_PROMPT: &str = include_str!("../../../prompts/context-summarizer-prompt.md");

/// Context summarizer client for condensing oversized evidence bundles (D-006).
#[derive(Debug, Clone)]
pub struct LlmSummarizer {
    endpoint: Option<String>,
    model_name: String,
    api_key: Option<String>,
    timeout: Duration,
    http_client: Client,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    response_format: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

impl LlmSummarizer {
    pub fn new(
        endpoint: Option<String>,
        model_name: String,
        api_key: Option<String>,
        timeout_ms: u64,
    ) -> Self {
        let timeout = Duration::from_millis(timeout_ms);
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();

        Self {
            endpoint,
            model_name,
            api_key,
            timeout,
            http_client,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Truncate and condense items deterministically offline.
    fn summarize_offline(
        &self,
        raw_bundle: &EvidenceBundle,
        _target_tokens: usize,
    ) -> EvidenceBundle {
        let mut condensed_items = Vec::new();

        for item in &raw_bundle.items {
            let condensed_content = if item.content.len() > 300 {
                format!(
                    "{}...\n[Condensed {} bytes to excerpt]",
                    &item.content[..250],
                    item.content.len()
                )
            } else {
                item.content.clone()
            };

            let mut summarized_item = item.clone();
            summarized_item.content = condensed_content;
            summarized_item.classification = Classification::DerivedSummary;
            condensed_items.push(summarized_item);
        }

        EvidenceBundle {
            template_id: raw_bundle.template_id,
            profile: raw_bundle.profile,
            items: condensed_items,
            status: raw_bundle.status,
        }
    }
}

#[async_trait]
impl ContextSummarizer for LlmSummarizer {
    async fn summarize(
        &self,
        raw_bundle: &EvidenceBundle,
        target_tokens: usize,
    ) -> Result<EvidenceBundle, CoreError> {
        if raw_bundle.is_empty() {
            return Ok(raw_bundle.clone());
        }

        let endpoint = match &self.endpoint {
            Some(ep) if !ep.is_empty() => ep,
            _ => return Ok(self.summarize_offline(raw_bundle, target_tokens)),
        };

        let raw_evidence_text = raw_bundle.format_model_facing();
        let req_body = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: SUMMARIZER_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Target Token Budget: {}\n\n{}",
                        target_tokens, raw_evidence_text
                    ),
                },
            ],
            temperature: 0.1,
            response_format: Some(json!({ "type": "json_object" })),
        };

        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
        let mut req = self.http_client.post(&url).json(&req_body);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.map_err(|e| {
            CoreError::ValidationFailed(format!("Summarizer request failed: {}", e))
        })?;

        if !resp.status().is_success() {
            return Ok(self.summarize_offline(raw_bundle, target_tokens));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CoreError::ValidationFailed(format!("Summarizer parse error: {}", e)))?;

        let content_str = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();

        let envelope: SummarizerEnvelope = match serde_json::from_str(content_str) {
            Ok(env) => env,
            Err(_) => return Ok(self.summarize_offline(raw_bundle, target_tokens)),
        };

        let mut summarized_items = Vec::new();
        for item in &raw_bundle.items {
            let new_content = envelope
                .summaries
                .iter()
                .find(|s| s.id == item.id)
                .map(|s| s.summary.clone())
                .unwrap_or_else(|| item.content.clone());

            let mut new_item = item.clone();
            new_item.content = new_content;
            new_item.classification = Classification::DerivedSummary;
            summarized_items.push(new_item);
        }

        Ok(EvidenceBundle {
            template_id: raw_bundle.template_id,
            profile: raw_bundle.profile,
            items: summarized_items,
            status: raw_bundle.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pe_context::{EvidenceItem, Provenance};
    use pe_core::templates::CollectionProfile;
    use pe_core::types::TemplateId;
    use pe_core::EvidenceStatus;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_summarizer_offline_condensation() {
        let summarizer = LlmSummarizer::new(None, "gpt-4o-mini".to_string(), None, 4000);
        assert_eq!(summarizer.timeout(), Duration::from_millis(4000));
        let bundle = EvidenceBundle {
            template_id: TemplateId::ImplementationRefactoring,
            profile: CollectionProfile::TargetedFile,
            items: vec![EvidenceItem {
                id: "ev_1".to_string(),
                content: "a".repeat(500),
                classification: Classification::DirectObservation,
                provenance: Provenance {
                    relative_path: PathBuf::from("src/main.rs"),
                    line_range: None,
                    sha256_hash: "hash".to_string(),
                    modified_at: None,
                },
                relevance_rationale: "Target file".to_string(),
                need: "context".to_string(),
            }],
            status: EvidenceStatus::SufficientContext,
        };

        let summarized = summarizer.summarize(&bundle, 100).await.unwrap();
        assert_eq!(summarized.items.len(), 1);
        assert_eq!(
            summarized.items[0].classification,
            Classification::DerivedSummary
        );
        assert!(summarized.items[0].content.contains("Condensed 500 bytes"));
    }
}
