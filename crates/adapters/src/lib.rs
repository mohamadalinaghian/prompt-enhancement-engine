//! External adapters (Jev, Prompt Writer, Summarizer) for Prompt Enhancement Engine.

pub mod envelope;
pub mod jev;
pub mod rules;
pub mod summarizer;
pub mod traits;
pub mod writer;

pub use envelope::{SummarizedItem, SummarizerEnvelope, WriterEnvelope};
pub use jev::JevClient;
pub use rules::RuleDecisionEngine;
pub use summarizer::LlmSummarizer;
pub use traits::{ContextSummarizer, DecisionProvider, DecisionResult, PromptWriter};
pub use writer::LlmWriter;
