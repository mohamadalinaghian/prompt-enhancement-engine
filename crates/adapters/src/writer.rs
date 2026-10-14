use crate::envelope::WriterEnvelope;
use crate::traits::PromptWriter;
use async_trait::async_trait;
use pe_context::EvidenceBundle;
use pe_core::templates::TemplateContract;
use pe_core::CoreError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

static MASTER_PROMPT: &str = include_str!("../../../prompts/master-prompt.md");

/// Generic LLM adapter for prompt enhancement and field population.
#[derive(Debug, Clone)]
pub struct LlmWriter {
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

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

impl LlmWriter {
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

    /// Build system instructions combining master prompt and template field contracts.
    fn build_system_prompt(&self, contract: &TemplateContract) -> String {
        let mut sys = MASTER_PROMPT.to_string();
        sys.push_str("\n\n## Target Template Contract\n");
        sys.push_str(&format!("Template: {}\n", contract.template_id.as_str()));
        sys.push_str(&format!("Session Kind: {:?}\n\n", contract.session_kind));
        sys.push_str("### Dynamic Fields to Populate:\n");

        for field in contract.dynamic_fields {
            sys.push_str(&format!(
                "- `{}` ({}, Policy: {:?}): {}\n",
                field.name,
                if field.required {
                    "REQUIRED"
                } else {
                    "OPTIONAL"
                },
                field.evidence_policy,
                field.description
            ));
        }

        sys
    }

    /// Build user prompt combining task text and formatted model-facing evidence.
    fn build_user_prompt(&self, task_text: &str, evidence: &EvidenceBundle) -> String {
        let mut user = format!("### Raw User Task Prompt:\n{}\n\n", task_text);
        if !evidence.is_empty() {
            user.push_str(&evidence.format_model_facing());
        }
        user
    }

    /// Deterministic offline field generator for testing and offline execution.
    pub fn generate_offline_fallback(
        task_text: &str,
        contract: &TemplateContract,
    ) -> WriterEnvelope {
        let mut fields = HashMap::new();
        fields.insert("task".to_string(), task_text.trim().to_string());
        fields.insert(
            "acceptance_criteria".to_string(),
            "- Execute task according to specifications\n- Verify all changes pass tests"
                .to_string(),
        );

        for field in contract.dynamic_fields {
            if !fields.contains_key(field.name) && field.required {
                fields.insert(
                    field.name.to_string(),
                    format!("Default context for {}", field.name),
                );
            }
        }

        WriterEnvelope::Ok { fields }
    }
}

#[async_trait]
impl PromptWriter for LlmWriter {
    async fn generate_fields(
        &self,
        task_text: &str,
        contract: &TemplateContract,
        evidence: &EvidenceBundle,
    ) -> Result<WriterEnvelope, CoreError> {
        let endpoint = match &self.endpoint {
            Some(ep) if !ep.is_empty() => ep,
            _ => return Ok(Self::generate_offline_fallback(task_text, contract)),
        };

        let system_prompt = self.build_system_prompt(contract);
        let user_prompt = self.build_user_prompt(task_text, evidence);

        let req_body = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: 0.2,
            response_format: Some(json!({ "type": "json_object" })),
        };

        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
        let mut req = self.http_client.post(&url).json(&req_body);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::ValidationFailed(format!("LLM request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(CoreError::ValidationFailed(format!(
                "LLM returned HTTP status {}",
                resp.status()
            )));
        }

        let completion: ChatCompletionResponse = resp.json().await.map_err(|e| {
            CoreError::ValidationFailed(format!("Failed to parse LLM response: {}", e))
        })?;

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| CoreError::ValidationFailed("LLM returned 0 choices".to_string()))?;

        let envelope: WriterEnvelope =
            serde_json::from_str(&choice.message.content).map_err(|e| {
                CoreError::ValidationFailed(format!(
                    "Malformed JSON in LLM response: {} (Body: {})",
                    e, choice.message.content
                ))
            })?;

        // Validate envelope against required fields
        if let WriterEnvelope::Ok { ref fields } = envelope {
            for req_field in contract.required_fields() {
                if !fields.contains_key(req_field.name) || fields[req_field.name].trim().is_empty()
                {
                    return Err(CoreError::TemplateContractMismatch(format!(
                        "LLM failed to populate required field '{}'",
                        req_field.name
                    )));
                }
            }
        }

        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pe_core::templates::definitions::IMPL_REFACTOR_CONTRACT;
    use pe_core::EvidenceStatus;

    #[tokio::test]
    async fn test_offline_fallback_generation() {
        let writer = LlmWriter::new(None, "gpt-4".to_string(), None, 5000);
        assert_eq!(writer.timeout(), Duration::from_millis(5000));
        let contract = &IMPL_REFACTOR_CONTRACT;
        let evidence = EvidenceBundle::empty(contract.template_id, EvidenceStatus::NoCollection);

        let res = writer
            .generate_fields("Add metrics endpoint", contract, &evidence)
            .await
            .unwrap();

        match res {
            WriterEnvelope::Ok { fields } => {
                assert!(fields.contains_key("task"));
                assert!(fields.contains_key("acceptance_criteria"));
                assert!(fields.contains_key("relevant_files"));
            }
            WriterEnvelope::NeedsClarification { .. } => panic!("Expected Ok envelope"),
        }
    }
}
