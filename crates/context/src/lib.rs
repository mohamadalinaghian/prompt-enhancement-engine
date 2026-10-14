//! Safe local context collection and repository inspection for Prompt Enhancement Engine.

pub mod budget;
pub mod collector;
pub mod evidence;
pub mod jail;
pub mod scanner;

pub use budget::BudgetTracker;
pub use collector::FieldDrivenCollector;
pub use evidence::{Classification, EvidenceBundle, EvidenceItem, Provenance};
pub use jail::ProjectJail;
pub use scanner::FileScanner;
