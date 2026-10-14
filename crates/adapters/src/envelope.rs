use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response envelope returned by the Prompt Writer LLM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WriterEnvelope {
    Ok {
        fields: HashMap<String, String>,
    },
    NeedsClarification {
        question: String,
        blocking_reason: String,
    },
}

/// An individual condensed evidence item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummarizedItem {
    pub id: String,
    pub summary: String,
}

/// Response envelope returned by the Context Summarizer LLM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummarizerEnvelope {
    pub summaries: Vec<SummarizedItem>,
}
