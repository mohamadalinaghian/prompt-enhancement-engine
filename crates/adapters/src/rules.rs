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

    /// Pure deterministic template classifier using weighted multi-factor intent scoring.
    pub fn classify_template(task_text: &str) -> TemplateId {
        let lower = task_text.to_lowercase();
        let trimmed = task_text.trim();

        // 1. Definite syntactical patterns (highest priority)
        if lower.starts_with("git ") || lower.starts_with("git-") {
            return TemplateId::GitVcs;
        }

        if lower.starts_with("continue to step")
            || lower.starts_with("proceed with step")
            || lower.starts_with("step ")
            || lower.starts_with("next step")
            || lower.contains("lgtm proceed")
            || lower.contains("approved continue")
        {
            return TemplateId::SessionProgression;
        }

        // Pure read-only questions without implementation commands
        if (trimmed.ends_with('?')
            || lower.starts_with("how ")
            || lower.starts_with("why ")
            || lower.starts_with("what ")
            || lower.starts_with("explain ")
            || lower.starts_with("where "))
            && !lower.contains("fix")
            && !lower.contains("implement")
            && !lower.contains("create")
            && !lower.contains("refactor")
            && !lower.contains("make ")
        {
            return TemplateId::QuestionConsultation;
        }

        // 2. Multi-factor weighted score calculation
        let mut scores: std::collections::HashMap<TemplateId, usize> =
            std::collections::HashMap::new();

        let mut add_score = |id: TemplateId, weight: usize| {
            *scores.entry(id).or_insert(0) += weight;
        };

        // --- DevOps & Deployment ---
        let devops_high = [
            "deploy",
            "docker",
            "kubernetes",
            "k8s",
            "ci/cd",
            "pipeline",
            "nginx",
            "release",
            "prod-release",
            "production",
            "worktree",
            "safeguard",
            "container",
            "helm",
            "terraform",
            "ansible",
            "make prod",
        ];
        for kw in devops_high {
            if lower.contains(kw) {
                add_score(TemplateId::DevopsDeployment, 4);
            }
        }
        if lower.contains("makefile") || lower.contains("dockerfile") || lower.contains("staging") {
            add_score(TemplateId::DevopsDeployment, 3);
        }

        // --- Git VCS ---
        let git_high = [
            "commit ",
            "create branch",
            "pull request",
            "create pr",
            "rebase ",
            "cherry-pick",
            "git stash",
            "merge branch",
            "squash",
        ];
        for kw in git_high {
            if lower.contains(kw) {
                add_score(TemplateId::GitVcs, 4);
            }
        }
        if lower.contains("worktree") || lower.contains("repository") || lower.contains("vcs") {
            add_score(TemplateId::GitVcs, 2);
        }

        // --- Debug & Bug Fix ---
        let debug_high = [
            "bug",
            "panic",
            "crash",
            "segfault",
            "broken",
            "exception",
            "stack trace",
            "null pointer",
            "undefined is not",
        ];
        for kw in debug_high {
            if lower.contains(kw) {
                add_score(TemplateId::DebugBugfix, 4);
            }
        }
        if lower.contains("fix") || lower.contains("error") || lower.contains("fails") {
            add_score(TemplateId::DebugBugfix, 3);
        }

        // --- Testing & Verification ---
        // Require explicit testing intent; do not trigger on incidental words like "when tests run"
        let test_high = [
            "unit test",
            "integration test",
            "e2e test",
            "test suite",
            "add test",
            "write test",
            "test coverage",
            "assert",
            "mock",
            "fixture",
            "testing",
        ];
        for kw in test_high {
            if lower.contains(kw) {
                add_score(TemplateId::TestingVerification, 4);
            }
        }
        if lower.starts_with("test ") || lower.contains("run tests") || lower.contains("tests for")
        {
            add_score(TemplateId::TestingVerification, 3);
        }

        // --- Plan & Architecture ---
        let plan_high = [
            "architecture",
            "system design",
            "roadmap",
            "break down into steps",
            "migration plan",
            "technical spec",
        ];
        for kw in plan_high {
            if lower.contains(kw) {
                add_score(TemplateId::PlanArchitecture, 4);
            }
        }
        if lower.starts_with("plan ") || lower.contains("design ") || lower.contains("break down") {
            add_score(TemplateId::PlanArchitecture, 3);
        }

        // --- Codebase Audit & Security Review ---
        let audit_high = [
            "audit",
            "security review",
            "vulnerability",
            "cve",
            "penetration",
            "inspect codebase",
            "threat model",
        ];
        for kw in audit_high {
            if lower.contains(kw) {
                add_score(TemplateId::CodebaseAudit, 4);
            }
        }

        // --- Report & Status Summary ---
        let report_high = [
            "status report",
            "progress report",
            "changelog",
            "summary of changes",
            "compile report",
            "summarize progress",
        ];
        for kw in report_high {
            if lower.contains(kw) {
                add_score(TemplateId::ReportSummary, 4);
            }
        }

        // --- Research & Feasibility ---
        let research_high = [
            "research feasibility",
            "compare libraries",
            "evaluate tool",
            "benchmark comparison",
            "trade-off analysis",
        ];
        for kw in research_high {
            if lower.contains(kw) {
                add_score(TemplateId::ResearchFeasibility, 4);
            }
        }
        if lower.contains("research") || lower.contains("feasibility") {
            add_score(TemplateId::ResearchFeasibility, 3);
        }

        // --- Configuration & Tooling ---
        let config_high = [
            "tooling",
            "plist",
            "setup cli",
            "linter config",
            "tsconfig",
            ".eslintrc",
            "cargo.toml",
            "pyproject.toml",
            "configure cargo",
        ];
        for kw in config_high {
            if lower.contains(kw) {
                add_score(TemplateId::ConfigTooling, 4);
            }
        }
        if lower.contains("config") || lower.contains("install ") {
            add_score(TemplateId::ConfigTooling, 2);
        }

        // --- Prompt Engineering ---
        let prompt_high = [
            "prompt template",
            "prompt engineering",
            "system prompt",
            "system instruction",
            "xml guardrails",
            "meta-prompt",
        ];
        for kw in prompt_high {
            if lower.contains(kw) {
                add_score(TemplateId::PromptEngineering, 4);
            }
        }
        if lower.contains("prompt") {
            add_score(TemplateId::PromptEngineering, 2);
        }

        // --- Documentation ---
        let doc_high = [
            "documentation",
            "readme",
            "api doc",
            "user guide",
            "docstrings",
            "mkdocs",
        ];
        for kw in doc_high {
            if lower.contains(kw) {
                add_score(TemplateId::Documentation, 4);
            }
        }
        if lower.contains("document") || lower.contains("guide") {
            add_score(TemplateId::Documentation, 2);
        }

        // --- Implementation & Refactoring ---
        let impl_high = [
            "refactor",
            "implement",
            "add feature",
            "create endpoint",
            "rewrite",
        ];
        for kw in impl_high {
            if lower.contains(kw) {
                add_score(TemplateId::ImplementationRefactoring, 4);
            }
        }
        if lower.contains("write ") || lower.contains("create ") || lower.contains("add ") {
            add_score(TemplateId::ImplementationRefactoring, 2);
        }

        // Pick highest scoring template
        if let Some((best_template, highest_score)) =
            scores.into_iter().max_by_key(|&(_, score)| score)
        {
            if highest_score > 0 {
                return best_template;
            }
        }

        // Fallback default
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
        // Test multi-intent prompt where "when tests start running" should NOT override DevOps release intent
        assert_eq!(
            RuleDecisionEngine::classify_template(
                "during release process with target make command make prod-release-full and also make edit in codes, can we have a safe guard when tests start runing, first create a separate worktree"
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
