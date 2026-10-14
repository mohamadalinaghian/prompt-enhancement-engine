use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The 15 standardized prompt templates (templates/README.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateId {
    ImplementationRefactoring,
    DebugBugfix,
    QuestionConsultation,
    PlanArchitecture,
    TestingVerification,
    CodebaseAudit,
    ReportSummary,
    ResearchFeasibility,
    DevopsDeployment,
    GitVcs,
    ConfigTooling,
    PromptEngineering,
    Documentation,
    SessionProgression,
    GeneralTask,
}

impl TemplateId {
    pub const ALL: [TemplateId; 15] = [
        TemplateId::ImplementationRefactoring,
        TemplateId::DebugBugfix,
        TemplateId::QuestionConsultation,
        TemplateId::PlanArchitecture,
        TemplateId::TestingVerification,
        TemplateId::CodebaseAudit,
        TemplateId::ReportSummary,
        TemplateId::ResearchFeasibility,
        TemplateId::DevopsDeployment,
        TemplateId::GitVcs,
        TemplateId::ConfigTooling,
        TemplateId::PromptEngineering,
        TemplateId::Documentation,
        TemplateId::SessionProgression,
        TemplateId::GeneralTask,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ImplementationRefactoring => "implementation_refactoring",
            Self::DebugBugfix => "debug_bugfix",
            Self::QuestionConsultation => "question_consultation",
            Self::PlanArchitecture => "plan_architecture",
            Self::TestingVerification => "testing_verification",
            Self::CodebaseAudit => "codebase_audit",
            Self::ReportSummary => "report_summary",
            Self::ResearchFeasibility => "research_feasibility",
            Self::DevopsDeployment => "devops_deployment",
            Self::GitVcs => "git_vcs",
            Self::ConfigTooling => "config_tooling",
            Self::PromptEngineering => "prompt_engineering",
            Self::Documentation => "documentation",
            Self::SessionProgression => "session_progression",
            Self::GeneralTask => "general_task",
        }
    }

    pub fn template_filename(&self) -> String {
        format!("{}.md", self.as_str())
    }

    pub fn session_kind(&self) -> SessionKind {
        match self {
            Self::QuestionConsultation
            | Self::PlanArchitecture
            | Self::CodebaseAudit
            | Self::ReportSummary
            | Self::ResearchFeasibility => SessionKind::ReadOnly,
            _ => SessionKind::WriteEnabled,
        }
    }
}

/// Operational classification for permissions and invariant rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionKind {
    ReadOnly,
    WriteEnabled,
}

/// Canonical, verified project root path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedProjectRoot {
    canonical_path: PathBuf,
}

impl ValidatedProjectRoot {
    pub fn new(canonical_path: PathBuf) -> Self {
        Self { canonical_path }
    }

    pub fn path(&self) -> &Path {
        &self.canonical_path
    }
}

/// Extracted opaque block (fenced code, logs) preserved byte-for-byte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueBlock {
    pub id: String,
    pub original_text: String,
}

/// Result of parsing the initial user input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedInput {
    pub raw_input: String,
    pub task_text: String,
    pub project_root: Option<ValidatedProjectRoot>,
    pub opaque_blocks: Vec<OpaqueBlock>,
}
