use serde::{Deserialize, Serialize};
use std::fmt;

/// Intermediate evidence-phase outcomes (architecture.md §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStatus {
    NoCollection,
    SufficientContext,
    NeedsClarification,
    BudgetExhausted,
    ProjectUnavailable,
    SafetyBlocked,
    CollectionFailed,
}

impl fmt::Display for EvidenceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCollection => write!(f, "NO_COLLECTION"),
            Self::SufficientContext => write!(f, "SUFFICIENT_CONTEXT"),
            Self::NeedsClarification => write!(f, "NEEDS_CLARIFICATION"),
            Self::BudgetExhausted => write!(f, "BUDGET_EXHAUSTED"),
            Self::ProjectUnavailable => write!(f, "PROJECT_UNAVAILABLE"),
            Self::SafetyBlocked => write!(f, "SAFETY_BLOCKED"),
            Self::CollectionFailed => write!(f, "COLLECTION_FAILED"),
        }
    }
}

/// Run-level terminal statuses (architecture.md §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalStatus {
    Rendered,
    ClarificationEmitted,
    Busy,
    StaleClarificationMarker,
    InputEmpty,
    ConfigInvalid,
    ProjectUnavailable,
    SafetyBlocked,
    SecretScanFailed,
    TemplateContractMismatch,
    BudgetExhaustedInsufficient,
    CollectionFailed,
    SummarizerFailed,
    WriterFailed,
    ValidationFailed,
    DeadlineExceeded,
}

impl fmt::Display for TerminalStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rendered => write!(f, "RENDERED"),
            Self::ClarificationEmitted => write!(f, "CLARIFICATION_EMITTED"),
            Self::Busy => write!(f, "BUSY"),
            Self::StaleClarificationMarker => write!(f, "STALE_CLARIFICATION_MARKER"),
            Self::InputEmpty => write!(f, "INPUT_EMPTY"),
            Self::ConfigInvalid => write!(f, "CONFIG_INVALID"),
            Self::ProjectUnavailable => write!(f, "PROJECT_UNAVAILABLE"),
            Self::SafetyBlocked => write!(f, "SAFETY_BLOCKED"),
            Self::SecretScanFailed => write!(f, "SECRET_SCAN_FAILED"),
            Self::TemplateContractMismatch => write!(f, "TEMPLATE_CONTRACT_MISMATCH"),
            Self::BudgetExhaustedInsufficient => write!(f, "BUDGET_EXHAUSTED_INSUFFICIENT"),
            Self::CollectionFailed => write!(f, "COLLECTION_FAILED"),
            Self::SummarizerFailed => write!(f, "SUMMARIZER_FAILED"),
            Self::WriterFailed => write!(f, "WRITER_FAILED"),
            Self::ValidationFailed => write!(f, "VALIDATION_FAILED"),
            Self::DeadlineExceeded => write!(f, "DEADLINE_EXCEEDED"),
        }
    }
}

/// Action classification for shell script integration (workflow-integration.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitClass {
    Success,
    Clarification,
    Busy,
    Failure,
}

impl TerminalStatus {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Rendered => 0,
            Self::ClarificationEmitted => 2,
            Self::Busy => 3,
            _ => 1,
        }
    }

    pub fn exit_class(&self) -> ExitClass {
        match self {
            Self::Rendered => ExitClass::Success,
            Self::ClarificationEmitted => ExitClass::Clarification,
            Self::Busy => ExitClass::Busy,
            _ => ExitClass::Failure,
        }
    }

    pub fn should_paste(&self) -> bool {
        matches!(self, Self::Rendered | Self::ClarificationEmitted)
    }
}

/// Operational flags emitted on stderr.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalFlags {
    pub jev_degraded: bool,
}

impl OperationalFlags {
    pub fn format_stderr(&self, status: TerminalStatus) -> String {
        if self.jev_degraded {
            format!("{status} JEV_DEGRADED")
        } else {
            status.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes_and_classes() {
        assert_eq!(TerminalStatus::Rendered.exit_code(), 0);
        assert_eq!(TerminalStatus::Rendered.exit_class(), ExitClass::Success);
        assert!(TerminalStatus::Rendered.should_paste());

        assert_eq!(TerminalStatus::ClarificationEmitted.exit_code(), 2);
        assert_eq!(
            TerminalStatus::ClarificationEmitted.exit_class(),
            ExitClass::Clarification
        );
        assert!(TerminalStatus::ClarificationEmitted.should_paste());

        assert_eq!(TerminalStatus::Busy.exit_code(), 3);
        assert_eq!(TerminalStatus::Busy.exit_class(), ExitClass::Busy);
        assert!(!TerminalStatus::Busy.should_paste());

        assert_eq!(TerminalStatus::InputEmpty.exit_code(), 1);
        assert_eq!(TerminalStatus::InputEmpty.exit_class(), ExitClass::Failure);
        assert!(!TerminalStatus::InputEmpty.should_paste());
    }

    #[test]
    fn test_operational_flags_stderr() {
        let mut flags = OperationalFlags::default();
        assert_eq!(flags.format_stderr(TerminalStatus::Rendered), "RENDERED");

        flags.jev_degraded = true;
        assert_eq!(
            flags.format_stderr(TerminalStatus::Rendered),
            "RENDERED JEV_DEGRADED"
        );
    }
}
