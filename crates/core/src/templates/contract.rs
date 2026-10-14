use crate::types::{SessionKind, TemplateId};
use serde::{Deserialize, Serialize};

/// Collection profiles allowed for this template (context-collection.md §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionProfile {
    None,
    TargetedFile,
    BugContext,
    ArchitectureContext,
    TestContext,
    ConfigurationContext,
}

/// Sourcing policy for a variable field (context-collection.md §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePolicy {
    /// Field must be derived exclusively from the user's input.
    UserTextOnly,
    /// Field may be informed by verified repository context.
    EvidenceAllowed,
}

/// Contract definition for a dynamic field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FieldContract {
    pub name: &'static str,
    pub required: bool,
    pub evidence_policy: EvidencePolicy,
    pub description: &'static str,
}

/// Hardcoded static sections for this template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StaticSections {
    pub rules: &'static str,
    pub verify: Option<&'static str>,
    pub report: &'static str,
    pub out_of_scope: &'static str,
}

/// Complete contract for one of the 15 prompt templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateContract {
    pub template_id: TemplateId,
    pub session_kind: SessionKind,
    pub allowed_profiles: &'static [CollectionProfile],
    pub dynamic_fields: &'static [FieldContract],
    pub static_sections: StaticSections,
}

impl TemplateContract {
    pub fn has_evidence_allowed_field(&self) -> bool {
        self.dynamic_fields
            .iter()
            .any(|f| f.evidence_policy == EvidencePolicy::EvidenceAllowed)
    }

    pub fn get_field(&self, name: &str) -> Option<&'static FieldContract> {
        self.dynamic_fields.iter().find(|f| f.name == name)
    }

    pub fn required_fields(&self) -> Vec<&'static FieldContract> {
        self.dynamic_fields.iter().filter(|f| f.required).collect()
    }
}
