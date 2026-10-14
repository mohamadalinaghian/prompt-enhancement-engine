use crate::envelope::WriterEnvelope;
use async_trait::async_trait;
use pe_context::EvidenceBundle;
use pe_core::templates::TemplateContract;
use pe_core::types::TemplateId;
use pe_core::CoreError;
use serde::{Deserialize, Serialize};

/// Result of a semantic routing or gatekeeping decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionResult<T> {
    pub value: T,
    pub confidence: f64,
    pub degraded: bool,
    pub source: &'static str,
}

impl<T> DecisionResult<T> {
    pub fn new(value: T, confidence: f64, degraded: bool, source: &'static str) -> Self {
        Self {
            value,
            confidence,
            degraded,
            source,
        }
    }
}

/// Provider for high-speed template routing and context necessity evaluation.
#[async_trait]
pub trait DecisionProvider: Send + Sync {
    /// Select the most appropriate template from the 15 available options.
    async fn select_template(
        &self,
        task_text: &str,
    ) -> Result<DecisionResult<TemplateId>, CoreError>;

    /// Decide whether project context collection is warranted.
    async fn should_collect_context(
        &self,
        task_text: &str,
        template_id: TemplateId,
    ) -> Result<DecisionResult<bool>, CoreError>;
}

/// Adapter for calling external Prompt Writer LLMs.
#[async_trait]
pub trait PromptWriter: Send + Sync {
    /// Generate variable field values or a clarification request.
    async fn generate_fields(
        &self,
        task_text: &str,
        contract: &TemplateContract,
        evidence: &EvidenceBundle,
    ) -> Result<WriterEnvelope, CoreError>;
}

/// Adapter for condensing oversized evidence bundles.
#[async_trait]
pub trait ContextSummarizer: Send + Sync {
    /// Condense raw evidence items into concise derived summaries.
    async fn summarize(
        &self,
        raw_bundle: &EvidenceBundle,
        target_tokens: usize,
    ) -> Result<EvidenceBundle, CoreError>;
}
