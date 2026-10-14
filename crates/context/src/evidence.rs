use chrono::{DateTime, Utc};
use pe_core::templates::CollectionProfile;
use pe_core::types::TemplateId;
use pe_core::EvidenceStatus;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Epistemic classification tier for an evidence item (evidence-bundle.md §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// Stated explicitly by the user in the task text.
    UserExplicit,
    /// Verified text or fact Rust read directly from a permitted local file.
    DirectObservation,
    /// Condensation produced by the summarizer model.
    DerivedSummary,
    /// Inferred logical conclusion based on evidence.
    Inference,
    /// Unverified statement or hypothesis.
    Unverified,
}

/// Provenance metadata grounding an evidence item (evidence-bundle.md §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub relative_path: PathBuf,
    pub line_range: Option<(usize, usize)>,
    pub sha256_hash: String,
    pub modified_at: Option<DateTime<Utc>>,
}

/// An individual verified evidence entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: String,
    pub content: String,
    pub classification: Classification,
    pub provenance: Provenance,
    pub relevance_rationale: String,
    pub need: String,
}

/// In-memory evidence bundle connecting collection to model adapters (evidence-bundle.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub template_id: TemplateId,
    pub profile: CollectionProfile,
    pub items: Vec<EvidenceItem>,
    pub status: EvidenceStatus,
}

impl EvidenceBundle {
    pub fn empty(template_id: TemplateId, status: EvidenceStatus) -> Self {
        Self {
            template_id,
            profile: CollectionProfile::None,
            items: Vec::new(),
            status,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Formats the sanitized model-facing evidence section (evidence-bundle.md §7).
    pub fn format_model_facing(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let mut out = String::from("### Verified Local Context (Untrusted Data):\n");
        for item in &self.items {
            let line_info = match item.provenance.line_range {
                Some((start, end)) => format!(":L{}-L{}", start, end),
                None => String::new(),
            };
            out.push_str(&format!(
                "- [{}] ({}{}) [{}]:\n```\n{}\n```\n\n",
                item.id,
                item.provenance.relative_path.display(),
                line_info,
                serde_json::to_string(&item.classification).unwrap_or_default(),
                item.content.trim()
            ));
        }
        out
    }
}
