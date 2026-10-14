//! External adapters (Jev, Prompt Writer, Summarizer) for Prompt Enhancement Engine.

pub fn adapters_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
