use crate::rules::RuleDecisionEngine;
use crate::traits::{DecisionProvider, DecisionResult};
use async_trait::async_trait;
use pe_core::types::TemplateId;
use pe_core::CoreError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_JEV_CONFIDENCE_THRESHOLD: f64 = 0.75;

/// Jev System One decision engine client with automatic rule degradation (D-010).
#[derive(Debug, Clone)]
pub struct JevClient {
    endpoint: Option<String>,
    api_key: Option<String>,
    confidence_threshold: f64,
    http_client: Client,
    rule_engine: RuleDecisionEngine,
}

#[derive(Serialize)]
struct JevChoiceRequest {
    prompt: String,
    options: Vec<String>,
}

#[derive(Deserialize)]
struct JevChoiceResponse {
    choice: String,
    score: f64,
}

#[derive(Serialize)]
struct JevNoulRequest {
    prompt: String,
    hypothesis: String,
}

#[derive(Deserialize)]
struct JevNoulResponse {
    probability: f64,
}

impl JevClient {
    pub fn new(endpoint: Option<String>, api_key: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_millis(500)) // Jev is fast (~100ms)
            .build()
            .unwrap_or_default();

        Self {
            endpoint,
            api_key,
            confidence_threshold: DEFAULT_JEV_CONFIDENCE_THRESHOLD,
            http_client,
            rule_engine: RuleDecisionEngine::new(),
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Internal call to Jev Choice for template classification.
    async fn call_jev_choice(
        &self,
        task_text: &str,
    ) -> Result<DecisionResult<TemplateId>, CoreError> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or_else(|| CoreError::SafetyBlocked("Jev endpoint not configured".to_string()))?;

        let options: Vec<String> = TemplateId::ALL
            .iter()
            .map(|t| t.as_str().to_string())
            .collect();

        let req_body = JevChoiceRequest {
            prompt: task_text.to_string(),
            options,
        };

        let url = format!("{}/classify_choice", endpoint.trim_end_matches('/'));
        let mut req = self.http_client.post(&url).json(&req_body);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::SafetyBlocked(format!("Jev Choice HTTP error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(CoreError::SafetyBlocked(format!(
                "Jev Choice returned status: {}",
                resp.status()
            )));
        }

        let jev_res: JevChoiceResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::SafetyBlocked(format!("Jev JSON parse error: {}", e)))?;

        // Match chosen string to TemplateId
        let matched_template = TemplateId::ALL
            .iter()
            .find(|t| t.as_str() == jev_res.choice)
            .copied()
            .ok_or_else(|| {
                CoreError::SafetyBlocked(format!("Unknown template returned: {}", jev_res.choice))
            })?;

        if jev_res.score < self.confidence_threshold {
            return Err(CoreError::SafetyBlocked(format!(
                "Jev score {} below threshold {}",
                jev_res.score, self.confidence_threshold
            )));
        }

        Ok(DecisionResult::new(
            matched_template,
            jev_res.score,
            false,
            "jev",
        ))
    }

    /// Internal call to Jev Noul for context necessity evaluation.
    async fn call_jev_noul(
        &self,
        task_text: &str,
        template_id: TemplateId,
    ) -> Result<DecisionResult<bool>, CoreError> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or_else(|| CoreError::SafetyBlocked("Jev endpoint not configured".to_string()))?;

        let req_body = JevNoulRequest {
            prompt: task_text.to_string(),
            hypothesis: format!(
                "This task requires inspecting local repository files for template {}",
                template_id.as_str()
            ),
        };

        let url = format!("{}/evaluate_noul", endpoint.trim_end_matches('/'));
        let mut req = self.http_client.post(&url).json(&req_body);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::SafetyBlocked(format!("Jev Noul HTTP error: {}", e)))?;

        if !resp.status().is_success() {
            return Err(CoreError::SafetyBlocked(format!(
                "Jev Noul returned status: {}",
                resp.status()
            )));
        }

        let jev_res: JevNoulResponse = resp
            .json()
            .await
            .map_err(|e| CoreError::SafetyBlocked(format!("Jev JSON parse error: {}", e)))?;

        let should_collect = jev_res.probability >= 0.5;
        Ok(DecisionResult::new(
            should_collect,
            jev_res.probability,
            false,
            "jev",
        ))
    }
}

#[async_trait]
impl DecisionProvider for JevClient {
    async fn select_template(
        &self,
        task_text: &str,
    ) -> Result<DecisionResult<TemplateId>, CoreError> {
        // Attempt Jev Choice
        match self.call_jev_choice(task_text).await {
            Ok(res) => Ok(res),
            Err(_) => {
                // Degrade gracefully to rule engine per D-010
                let mut fallback = self.rule_engine.select_template(task_text).await?;
                fallback.degraded = true;
                fallback.source = "rules_fallback";
                Ok(fallback)
            }
        }
    }

    async fn should_collect_context(
        &self,
        task_text: &str,
        template_id: TemplateId,
    ) -> Result<DecisionResult<bool>, CoreError> {
        // Attempt Jev Noul
        match self.call_jev_noul(task_text, template_id).await {
            Ok(res) => Ok(res),
            Err(_) => {
                // Degrade gracefully to rule engine per D-010
                let mut fallback = self
                    .rule_engine
                    .should_collect_context(task_text, template_id)
                    .await?;
                fallback.degraded = true;
                fallback.source = "rules_fallback";
                Ok(fallback)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jev_client_graceful_degradation_when_offline() {
        // Non-existent or invalid endpoint must degrade to rules with degraded = true
        let client = JevClient::new(Some("http://127.0.0.1:9".to_string()), None);
        let result = client
            .select_template("Fix the broken panic in main.rs")
            .await
            .unwrap();

        assert_eq!(result.value, TemplateId::DebugBugfix);
        assert!(result.degraded);
        assert_eq!(result.source, "rules_fallback");
    }

    #[tokio::test]
    async fn test_jev_client_context_necessity_degradation() {
        let client = JevClient::new(None, None);
        let result = client
            .should_collect_context(
                "Refactor src/cache.rs",
                TemplateId::ImplementationRefactoring,
            )
            .await
            .unwrap();

        assert!(result.value);
        assert!(result.degraded);
    }
}
