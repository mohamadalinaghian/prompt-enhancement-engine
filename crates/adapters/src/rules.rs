use crate::traits::{DecisionProvider, DecisionResult};
use async_trait::async_trait;
use pe_core::templates::TemplateContract;
use pe_core::types::TemplateId;
use pe_core::CoreError;

/// Deterministic, zero-latency rule-based decision fallback engine (D-010).
#[derive(Debug, Default, Clone, Copy)]
pub struct RuleDecisionEngine;

impl RuleDecisionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Pure deterministic template classifier.
    pub fn classify_template(task_text: &str) -> TemplateId {
        let lower = task_text.to_lowercase();
        let trimmed = task_text.trim();

        // 1. Git operations
        if lower.starts_with("git ")
            || lower.contains("commit ")
            || lower.contains("create branch")
            || lower.contains("pull request")
            || lower.contains("create pr")
            || lower.contains("rebase ")
            || lower.contains("cherry-pick")
        {
            return TemplateId::GitVcs;
        }

        // 2. Session progression (multi-step continuation)
        if lower.starts_with("continue to step")
            || lower.starts_with("step ")
            || lower.starts_with("proceed with step")
            || lower.contains("lgtm proceed")
            || lower.contains("approved continue")
        {
            return TemplateId::SessionProgression;
        }

        // 3. Question & Consultation (read-only explanation)
        if (trimmed.ends_with('?')
            || lower.starts_with("how ")
            || lower.starts_with("why ")
            || lower.starts_with("what ")
            || lower.starts_with("explain ")
            || lower.starts_with("where "))
            && !lower.contains("fix")
            && !lower.contains("implement")
        {
            return TemplateId::QuestionConsultation;
        }

        // 4. Debug & Bug Fix
        if lower.contains("bug")
            || lower.contains("fix")
            || lower.contains("error")
            || lower.contains("panic")
            || lower.contains("crash")
            || lower.contains("fails")
            || lower.contains("broken")
            || lower.contains("exception")
        {
            return TemplateId::DebugBugfix;
        }

        // 5. Testing & Verification
        if lower.contains("test")
            || lower.contains("unit test")
            || lower.contains("integration test")
            || lower.contains("coverage")
            || lower.contains("assert")
        {
            return TemplateId::TestingVerification;
        }

        // 6. Plan & Architecture
        if lower.contains("plan ")
            || lower.contains("architecture")
            || lower.contains("design ")
            || lower.contains("roadmap")
            || lower.contains("break down")
        {
            return TemplateId::PlanArchitecture;
        }

        // 7. Codebase Audit & Inspection
        if lower.contains("audit")
            || lower.contains("security review")
            || lower.contains("vulnerability")
            || lower.contains("inspect codebase")
        {
            return TemplateId::CodebaseAudit;
        }

        // 8. Report & Status Summary
        if lower.contains("status report")
            || lower.contains("progress report")
            || lower.contains("changelog")
            || lower.contains("summary of changes")
        {
            return TemplateId::ReportSummary;
        }

        // 9. Research & Feasibility
        if lower.contains("research")
            || lower.contains("feasibility")
            || lower.contains("compare libraries")
            || lower.contains("evaluate tool")
        {
            return TemplateId::ResearchFeasibility;
        }

        // 10. DevOps & Deployment
        if lower.contains("deploy")
            || lower.contains("docker")
            || lower.contains("kubernetes")
            || lower.contains("k8s")
            || lower.contains("ci/cd")
            || lower.contains("pipeline")
            || lower.contains("nginx")
        {
            return TemplateId::DevopsDeployment;
        }

        // 11. Configuration & Tooling
        if lower.contains("config")
            || lower.contains("tooling")
            || lower.contains("plist")
            || lower.contains("setup cli")
            || lower.contains("install ")
        {
            return TemplateId::ConfigTooling;
        }

        // 12. Prompt Engineering
        if lower.contains("prompt")
            || lower.contains("system instruction")
            || lower.contains("template formatting")
        {
            return TemplateId::PromptEngineering;
        }

        // 13. Documentation
        if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("guide")
            || lower.contains("docstrings")
        {
            return TemplateId::Documentation;
        }

        // 14. Implementation & Refactoring
        if lower.contains("refactor")
            || lower.contains("implement")
            || lower.contains("add feature")
            || lower.contains("write ")
            || lower.contains("create endpoint")
        {
            return TemplateId::ImplementationRefactoring;
        }

        // 15. General Task fallback
        TemplateId::GeneralTask
    }

    /// Pure deterministic context necessity decider.
    pub fn evaluate_context_necessity(task_text: &str, template_id: TemplateId) -> bool {
        let contract = TemplateContract::for_template(template_id);
        if !contract.has_evidence_allowed_field() {
            return false;
        }

        let lower = task_text.to_lowercase();
        // If task references file extensions, paths, codebase symbols, or explicitly asks for context
        lower.contains(".rs")
            || lower.contains(".ts")
            || lower.contains(".py")
            || lower.contains(".toml")
            || lower.contains(".json")
            || lower.contains('/')
            || lower.contains("file")
            || lower.contains("code")
            || lower.contains("function")
            || lower.contains("module")
            || lower.contains("crate")
            || lower.contains("repo")
    }
}

#[async_trait]
impl DecisionProvider for RuleDecisionEngine {
    async fn select_template(
        &self,
        task_text: &str,
    ) -> Result<DecisionResult<TemplateId>, CoreError> {
        let template_id = Self::classify_template(task_text);
        Ok(DecisionResult::new(template_id, 1.0, false, "rules"))
    }

    async fn should_collect_context(
        &self,
        task_text: &str,
        template_id: TemplateId,
    ) -> Result<DecisionResult<bool>, CoreError> {
        let should_collect = Self::evaluate_context_necessity(task_text, template_id);
        Ok(DecisionResult::new(should_collect, 1.0, false, "rules"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_classification_scenarios() {
        assert_eq!(
            RuleDecisionEngine::classify_template("git commit -m 'feat: add auth'"),
            TemplateId::GitVcs
        );
        assert_eq!(
            RuleDecisionEngine::classify_template("Fix the null pointer panic in parser.rs"),
            TemplateId::DebugBugfix
        );
        assert_eq!(
            RuleDecisionEngine::classify_template("How does single-flight locking work in Rust?"),
            TemplateId::QuestionConsultation
        );
        assert_eq!(
            RuleDecisionEngine::classify_template("Refactor the cache module to use dashmap"),
            TemplateId::ImplementationRefactoring
        );
        assert_eq!(
            RuleDecisionEngine::classify_template("Add unit tests for the token bucket"),
            TemplateId::TestingVerification
        );
        assert_eq!(
            RuleDecisionEngine::classify_template("Plan the migration to PostgreSQL"),
            TemplateId::PlanArchitecture
        );
        assert_eq!(
            RuleDecisionEngine::classify_template(
                "Audit the authentication handler for vulnerabilities"
            ),
            TemplateId::CodebaseAudit
        );
        assert_eq!(
            RuleDecisionEngine::classify_template(
                "Deploy the worker container with docker compose"
            ),
            TemplateId::DevopsDeployment
        );
    }

    #[test]
    fn test_context_necessity_rules() {
        assert!(RuleDecisionEngine::evaluate_context_necessity(
            "Fix bug in src/main.rs",
            TemplateId::DebugBugfix
        ));
        assert!(!RuleDecisionEngine::evaluate_context_necessity(
            "Commit files to git",
            TemplateId::GitVcs
        ));
    }
}
