use pe_cli::pipeline::execute_pipeline;
use pe_core::config::Config;
use pe_core::types::TemplateId;
use pe_core::TerminalStatus;

struct GoldenScenario {
    pub name: &'static str,
    pub input: &'static str,
    pub expected_template: TemplateId,
    pub expect_verify: bool,
}

#[tokio::test]
async fn test_all_15_golden_prompts() {
    let scenarios = [
        GoldenScenario {
            name: "ImplementationRefactoring",
            input: "Refactor auth middleware to use dashmap for caching tokens",
            expected_template: TemplateId::ImplementationRefactoring,
            expect_verify: true,
        },
        GoldenScenario {
            name: "DebugBugfix",
            input: "Fix panic on line 120 in parser.rs when parsing empty brackets",
            expected_template: TemplateId::DebugBugfix,
            expect_verify: true,
        },
        GoldenScenario {
            name: "QuestionConsultation",
            input: "How does the single-flight locking mechanism prevent races?",
            expected_template: TemplateId::QuestionConsultation,
            expect_verify: false, // Omitted for read-only consultation
        },
        GoldenScenario {
            name: "PlanArchitecture",
            input: "Plan the architecture migration of user storage from SQLite to PostgreSQL",
            expected_template: TemplateId::PlanArchitecture,
            expect_verify: true,
        },
        GoldenScenario {
            name: "TestingVerification",
            input: "Add unit tests for the token bucket rate limiter with boundary conditions",
            expected_template: TemplateId::TestingVerification,
            expect_verify: true,
        },
        GoldenScenario {
            name: "CodebaseAudit",
            input: "Audit the cryptographic routines in crypto.rs for security vulnerabilities",
            expected_template: TemplateId::CodebaseAudit,
            expect_verify: true,
        },
        GoldenScenario {
            name: "ReportSummary",
            input: "Compile a status report on the Phase 2 milestone deliverables and blockers",
            expected_template: TemplateId::ReportSummary,
            expect_verify: false, // Omitted for report summary
        },
        GoldenScenario {
            name: "ResearchFeasibility",
            input: "Research feasibility of migrating from tokio to monoio for event loop",
            expected_template: TemplateId::ResearchFeasibility,
            expect_verify: true,
        },
        GoldenScenario {
            name: "DevopsDeployment",
            input: "Deploy redis container with persistent volume and docker compose",
            expected_template: TemplateId::DevopsDeployment,
            expect_verify: true,
        },
        GoldenScenario {
            name: "GitVcs",
            input: "git commit staged changes with conventional commit format",
            expected_template: TemplateId::GitVcs,
            expect_verify: true,
        },
        GoldenScenario {
            name: "ConfigTooling",
            input: "Configure cargo tooling with all-targets and deny warnings in CI",
            expected_template: TemplateId::ConfigTooling,
            expect_verify: true,
        },
        GoldenScenario {
            name: "PromptEngineering",
            input: "Author a prompt template for code review agents with XML guardrails",
            expected_template: TemplateId::PromptEngineering,
            expect_verify: true,
        },
        GoldenScenario {
            name: "Documentation",
            input: "Update README and documentation with new multi-crate layout instructions",
            expected_template: TemplateId::Documentation,
            expect_verify: true,
        },
        GoldenScenario {
            name: "SessionProgression",
            input: "Continue to step 3: implement the parser module",
            expected_template: TemplateId::SessionProgression,
            expect_verify: true,
        },
        GoldenScenario {
            name: "GeneralTask",
            input: "Organize project directory and clean up temporary scratch files",
            expected_template: TemplateId::GeneralTask,
            expect_verify: true,
        },
    ];

    let config = Config::default();

    for scenario in scenarios {
        let outcome = execute_pipeline(scenario.input, &config, true).await;

        assert_eq!(
            outcome.status,
            TerminalStatus::Rendered,
            "Scenario '{}' failed with status {:?}",
            scenario.name,
            outcome.status
        );

        let contract =
            pe_core::templates::TemplateContract::for_template(scenario.expected_template);
        assert_eq!(contract.template_id, scenario.expected_template);

        let payload = outcome
            .output_payload
            .unwrap_or_else(|| panic!("Scenario '{}' produced no payload", scenario.name));

        // Required tag presence
        assert!(
            payload.contains(&format!(
                "<task session=\"{}\">",
                scenario.expected_template.as_str()
            )),
            "Scenario '{}' missing <task session=\"...\">",
            scenario.name
        );
        assert!(
            payload.contains("</task>"),
            "Scenario '{}' missing </task>",
            scenario.name
        );
        assert!(
            payload.contains("<acceptance_criteria>"),
            "Scenario '{}' missing <acceptance_criteria>",
            scenario.name
        );
        assert!(
            payload.contains("<scope>"),
            "Scenario '{}' missing <scope>",
            scenario.name
        );
        assert!(
            payload.contains("<rules>"),
            "Scenario '{}' missing <rules>",
            scenario.name
        );
        assert!(
            payload.contains("<report>"),
            "Scenario '{}' missing <report>",
            scenario.name
        );

        if scenario.expect_verify {
            assert!(
                payload.contains("<verify>"),
                "Scenario '{}' expected <verify> tag",
                scenario.name
            );
        } else {
            assert!(
                !payload.contains("<verify>"),
                "Scenario '{}' should NOT have <verify> tag",
                scenario.name
            );
        }
    }
}
