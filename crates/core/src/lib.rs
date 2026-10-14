//! Core domain logic, models, configuration, and security primitives for Prompt Enhancement Engine.

pub mod config;
pub mod error;
pub mod parser;
pub mod render;
pub mod secrets;
pub mod status;
pub mod templates;
pub mod types;

pub use config::Config;
pub use error::CoreError;
pub use parser::{InputParser, CLARIFY_BEGIN_SENTINEL, CLARIFY_END_SENTINEL, CLARIFY_HEADER};
pub use render::XmlRenderer;
pub use secrets::{SecretMap, SecretScanner};
pub use status::{EvidenceStatus, ExitClass, OperationalFlags, TerminalStatus};
pub use templates::{
    CollectionProfile, EvidencePolicy, FieldContract, StaticSections, TemplateContract,
};
pub use types::{OpaqueBlock, ParsedInput, SessionKind, TemplateId, ValidatedProjectRoot};
