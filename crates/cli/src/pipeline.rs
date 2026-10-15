use crate::clarification::format_clarification_block;
use pe_adapters::{
    ContextSummarizer, DecisionProvider, JevClient, LlmSummarizer, LlmWriter, PromptWriter,
    RuleDecisionEngine, WriterEnvelope,
};
use pe_context::{EvidenceBundle, FieldDrivenCollector};
use pe_core::config::Config;
use pe_core::parser::InputParser;
use pe_core::render::XmlRenderer;
use pe_core::secrets::SecretScanner;
use pe_core::templates::TemplateContract;
use pe_core::{CoreError, EvidenceStatus, OperationalFlags, TerminalStatus};

/// Result of executing the 14-step enhancement pipeline.
#[derive(Debug, Clone)]
pub struct PipelineOutcome {
    pub status: TerminalStatus,
    pub flags: OperationalFlags,
    pub output_payload: Option<String>,
}

impl PipelineOutcome {
    pub fn failure(status: TerminalStatus) -> Self {
        Self {
            status,
            flags: OperationalFlags::default(),
            output_payload: None,
        }
    }
}

/// Execute the full 14-step enhancement pipeline (architecture.md §3).
pub async fn execute_pipeline(
    raw_input: &str,
    config: &Config,
    force_no_llm: bool,
) -> PipelineOutcome {
    let mut flags = OperationalFlags::default();

    // 1. Input Parsing & Validation
    let parsed = match InputParser::parse(raw_input, config.budgets.max_input_prompt_bytes) {
        Ok(p) => p,
        Err(CoreError::InputEmpty) => return PipelineOutcome::failure(TerminalStatus::InputEmpty),
        Err(CoreError::StaleClarificationMarker) => {
            return PipelineOutcome::failure(TerminalStatus::StaleClarificationMarker)
        }
        Err(CoreError::ProjectUnavailable(_)) => {
            return PipelineOutcome::failure(TerminalStatus::ProjectUnavailable)
        }
        Err(CoreError::InputTooLarge { .. }) => {
            return PipelineOutcome::failure(TerminalStatus::InputEmpty)
        }
        Err(_) => return PipelineOutcome::failure(TerminalStatus::SafetyBlocked),
    };

    // 2. Secret Redaction on User Task Text (D-016)
    let (redacted_task, secret_map) = SecretScanner::redact(&parsed.task_text);

    // 3. Template Selection (Jev Choice with Rule Fallback, D-010)
    let (template_id, should_collect) = if force_no_llm {
        let rules = RuleDecisionEngine::new();
        let t_res = match rules.select_template(&redacted_task).await {
            Ok(res) => res,
            Err(_) => return PipelineOutcome::failure(TerminalStatus::ValidationFailed),
        };
        let c_res = match rules
            .should_collect_context(&redacted_task, t_res.value)
            .await
        {
            Ok(res) => res,
            Err(_) => return PipelineOutcome::failure(TerminalStatus::ValidationFailed),
        };
        (t_res.value, c_res.value)
    } else {
        let jev = JevClient::new(None, None); // Connects to local Jev daemon or degrades to rules
        let t_res = match jev.select_template(&redacted_task).await {
            Ok(res) => res,
            Err(_) => return PipelineOutcome::failure(TerminalStatus::ValidationFailed),
        };
        if t_res.degraded {
            flags.jev_degraded = true;
        }

        let c_res = match jev
            .should_collect_context(&redacted_task, t_res.value)
            .await
        {
            Ok(res) => res,
            Err(_) => return PipelineOutcome::failure(TerminalStatus::ValidationFailed),
        };
        if c_res.degraded {
            flags.jev_degraded = true;
        }

        (t_res.value, c_res.value)
    };

    let contract = TemplateContract::for_template(template_id);

    // 4. Safe Context Collection (D-005, D-018)
    let evidence_bundle = if should_collect && parsed.project_root.is_some() {
        match FieldDrivenCollector::collect(
            contract,
            &redacted_task,
            parsed.project_root.as_ref(),
            config,
        ) {
            Ok(bundle) => bundle,
            Err(CoreError::SafetyBlocked(_)) => {
                return PipelineOutcome::failure(TerminalStatus::SafetyBlocked)
            }
            Err(CoreError::ProjectUnavailable(_)) => {
                return PipelineOutcome::failure(TerminalStatus::ProjectUnavailable)
            }
            Err(_) => return PipelineOutcome::failure(TerminalStatus::CollectionFailed),
        }
    } else {
        EvidenceBundle::empty(template_id, EvidenceStatus::NoCollection)
    };

    // 5. Context Summarization when over budget (D-006)
    let final_evidence = if !evidence_bundle.is_empty() && !force_no_llm {
        let summarizer = LlmSummarizer::new(
            config.summarizer_model.endpoint.clone(),
            config.summarizer_model.model_name.clone(),
            std::env::var(&config.summarizer_model.api_key_env).ok(),
            config.budgets.writer_timeout_ms,
        );
        summarizer
            .summarize(&evidence_bundle, config.budgets.max_payload_tokens)
            .await
            .unwrap_or(evidence_bundle)
    } else {
        evidence_bundle
    };

    // 6. Prompt Writer LLM Field Population with Fallback Chain (D-009)
    let envelope = if force_no_llm {
        LlmWriter::generate_offline_fallback(&redacted_task, contract)
    } else {
        let mut model_chain = vec![config.writer_model.clone()];
        model_chain.extend(config.writer_model.fallback_models.clone());

        let mut candidate_result = None;

        for model_cfg in model_chain {
            let api_key = std::env::var(&model_cfg.api_key_env).ok();
            let writer = LlmWriter::new(
                model_cfg.endpoint.clone(),
                model_cfg.model_name.clone(),
                api_key,
                config.budgets.writer_timeout_ms,
            );

            match writer
                .generate_fields(&redacted_task, contract, &final_evidence)
                .await
            {
                Ok(env) => {
                    candidate_result = Some(env);
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "Model {} ({:?}) failed: {}. Trying next fallback...",
                        model_cfg.model_name, model_cfg.endpoint, e
                    );
                }
            }
        }

        match candidate_result {
            Some(env) => env,
            None => {
                // If all configured chain models fail or no endpoints configured, fallback to offline generator
                LlmWriter::generate_offline_fallback(&redacted_task, contract)
            }
        }
    };

    // 7. Handle Result Envelope & Render Deterministic XML
    match envelope {
        WriterEnvelope::NeedsClarification {
            question,
            blocking_reason,
        } => {
            let block = format_clarification_block(&question, &blocking_reason, raw_input);
            PipelineOutcome {
                status: TerminalStatus::ClarificationEmitted,
                flags,
                output_payload: Some(block),
            }
        }
        WriterEnvelope::Ok { fields } => {
            // Render XML structure in Rust (D-009)
            let rendered_xml = match XmlRenderer::render(contract, &fields) {
                Ok(xml) => xml,
                Err(_) => return PipelineOutcome::failure(TerminalStatus::ValidationFailed),
            };

            // Restore opaque blocks verbatim
            let with_opaque =
                InputParser::restore_opaque_blocks(&rendered_xml, &parsed.opaque_blocks);

            // Restore user secrets from placeholders (D-016)
            let final_prompt = SecretScanner::restore(&with_opaque, &secret_map);

            PipelineOutcome {
                status: TerminalStatus::Rendered,
                flags,
                output_payload: Some(final_prompt),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_empty_input() {
        let config = Config::default();
        let outcome = execute_pipeline("", &config, true).await;
        assert_eq!(outcome.status, TerminalStatus::InputEmpty);
        assert!(outcome.output_payload.is_none());
    }

    #[tokio::test]
    async fn test_pipeline_stale_clarification() {
        let config = Config::default();
        let input =
            "⚠ PROMPT ENHANCER — CLARIFICATION NEEDED\n[[PE-CLARIFY-v1:BEGIN]]\nQuestion: foo";
        let outcome = execute_pipeline(input, &config, true).await;
        assert_eq!(outcome.status, TerminalStatus::StaleClarificationMarker);
        assert!(outcome.output_payload.is_none());
    }

    #[tokio::test]
    async fn test_pipeline_offline_enhancement_success() {
        let config = Config::default();
        let input = "Fix the login route crash in auth.rs when token is expired";
        let outcome = execute_pipeline(input, &config, true).await;

        assert_eq!(outcome.status, TerminalStatus::Rendered);
        assert!(outcome.output_payload.is_some());
        let output = outcome.output_payload.unwrap();

        assert!(output.starts_with("<task>"));
        assert!(output.contains("<acceptance_criteria>"));
        assert!(output.contains("<rules>"));
        assert!(output.contains("<report>"));
    }

    #[tokio::test]
    async fn test_pipeline_secret_round_trip_restoration() {
        let config = Config::default();
        let secret = "sk-1234567890abcdef1234567890";
        let input = format!("Update API client using key {}", secret);

        let outcome = execute_pipeline(&input, &config, true).await;
        assert_eq!(outcome.status, TerminalStatus::Rendered);
        let output = outcome.output_payload.unwrap();

        // The secret must be restored in the output
        assert!(output.contains(secret));
        assert!(!output.contains("[[SECRET_PE_"));
    }
}
