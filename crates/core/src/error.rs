use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CoreError {
    #[error("Input text was empty or whitespace only")]
    InputEmpty,

    #[error("Input text contains stale clarification markers")]
    StaleClarificationMarker,

    #[error("Project path was not found or inaccessible: {0}")]
    ProjectUnavailable(String),

    #[error("Security rule violation: {0}")]
    SafetyBlocked(String),

    #[error("Secret scan failed: {0}")]
    SecretScanFailed(String),

    #[error("Configuration is invalid: {0}")]
    ConfigInvalid(String),

    #[error("Template contract mismatch: {0}")]
    TemplateContractMismatch(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Input exceeds maximum allowed size ({actual} bytes > {max} bytes)")]
    InputTooLarge { actual: usize, max: usize },
}
