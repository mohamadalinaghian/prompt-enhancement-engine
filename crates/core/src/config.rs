use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub total_deadline_ms: u64,
    pub writer_timeout_ms: u64,
    pub max_inspected_files: usize,
    pub max_read_bytes: usize,
    pub max_payload_tokens: usize,
    pub max_collection_iterations: usize,
    pub max_input_prompt_bytes: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            total_deadline_ms: 10_000,
            writer_timeout_ms: 6_000,
            max_inspected_files: 10,
            max_read_bytes: 128 * 1024,
            max_payload_tokens: 2_500,
            max_collection_iterations: 3,
            max_input_prompt_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionConfig {
    pub ignored_directories: Vec<String>,
    pub ignored_file_patterns: Vec<String>,
}

impl Default for ExclusionConfig {
    fn default() -> Self {
        Self {
            ignored_directories: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "vendor".to_string(),
            ],
            ignored_file_patterns: vec![
                ".env*".to_string(),
                "*.pem".to_string(),
                "*.key".to_string(),
                "*.p12".to_string(),
                "*.cert".to_string(),
                "*.crt".to_string(),
                "*.exe".to_string(),
                "*.dylib".to_string(),
                "*.so".to_string(),
                "*.bin".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub endpoint: Option<String>,
    pub model_name: String,
    pub api_key_env: String,
    #[serde(default)]
    pub fallback_models: Vec<ModelConfig>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "generic".to_string(),
            endpoint: None,
            model_name: "default".to_string(),
            api_key_env: "LLM_API_KEY".to_string(),
            fallback_models: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub budgets: BudgetConfig,
    #[serde(default)]
    pub exclusions: ExclusionConfig,
    #[serde(default)]
    pub writer_model: ModelConfig,
    #[serde(default)]
    pub summarizer_model: ModelConfig,
}

impl Config {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn load_or_default(config_path: Option<&Path>) -> Self {
        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(cfg) = Self::from_toml(&content) {
                        return cfg;
                    }
                }
            }
        }

        if let Some(mut home_cfg) = dirs::home_dir() {
            home_cfg.push(".config/prompt_enhancer/config.toml");
            if home_cfg.exists() {
                if let Ok(content) = std::fs::read_to_string(&home_cfg) {
                    if let Ok(cfg) = Self::from_toml(&content) {
                        return cfg;
                    }
                }
            }
        }

        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_budgets() {
        let cfg = Config::default();
        assert_eq!(cfg.budgets.total_deadline_ms, 10_000);
        assert_eq!(cfg.budgets.writer_timeout_ms, 6_000);
        assert_eq!(cfg.budgets.max_inspected_files, 10);
        assert_eq!(cfg.budgets.max_read_bytes, 128 * 1024);
        assert_eq!(cfg.budgets.max_payload_tokens, 2_500);
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
        [budgets]
        total_deadline_ms = 8000
        writer_timeout_ms = 5000
        max_inspected_files = 5
        max_read_bytes = 65536
        max_payload_tokens = 2000
        max_collection_iterations = 2
        max_input_prompt_bytes = 32768
        "#;
        let cfg = Config::from_toml(toml_str).expect("Valid TOML");
        assert_eq!(cfg.budgets.total_deadline_ms, 8_000);
        assert_eq!(cfg.budgets.max_inspected_files, 5);
        assert_eq!(cfg.budgets.max_collection_iterations, 2);
    }
}
