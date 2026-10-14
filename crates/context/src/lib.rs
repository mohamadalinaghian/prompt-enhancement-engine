//! Context collection and repository inspection for Prompt Enhancement Engine.

pub fn context_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
