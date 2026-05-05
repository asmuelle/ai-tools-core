use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Unified project configuration file (`.cargo-vibe.toml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VibeConfig {
    /// Global defaults shared by all tools.
    #[serde(default)]
    pub vibe: VibeGlobalConfig,
    /// `diff-risk` overrides.
    #[serde(default)]
    pub diff_risk: VibeToolConfig,
    /// `cargo-impact` overrides.
    #[serde(default)]
    pub cargo_impact: VibeToolConfig,
    /// `spec-drift` overrides.
    #[serde(default)]
    pub spec_drift: VibeToolConfig,
    /// `cargo-context` overrides.
    #[serde(default)]
    pub cargo_context: VibeToolConfig,
}

/// Global `.cargo-vibe.toml` settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VibeGlobalConfig {
    /// Default token budget for context-producing tools.
    #[serde(default)]
    pub token_budget: Option<usize>,
    /// Tokenizer identifier used for context budgeting.
    #[serde(default)]
    pub tokenizer: Option<String>,
    /// Whether secret scrubbing is enabled by default.
    #[serde(default)]
    pub scrub: Option<bool>,
    /// Default risk threshold.
    #[serde(default)]
    pub threshold: Option<f32>,
}

/// Per-tool configuration overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VibeToolConfig {
    /// Whether the tool is enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Tool-specific threshold override.
    #[serde(default)]
    pub threshold: Option<f32>,
    /// Extra CLI arguments passed to the tool.
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Load the project configuration, checking both `.cargo-vibe.toml`
/// and tool-specific config files as fallback.
pub fn load_project_config(start_dir: &Path) -> Option<VibeConfig> {
    let mut current = start_dir.to_path_buf();
    loop {
        let config_path = current.join(".cargo-vibe.toml");
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return Some(config),
                    Err(e) => {
                        eprintln!("cargo-vibe: failed to parse {}: {e}", config_path.display());
                        return None;
                    }
                },
                Err(e) => {
                    eprintln!("cargo-vibe: failed to read {}: {e}", config_path.display());
                    return None;
                }
            }
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Find the project root by walking up to find Cargo.toml or .git.
pub fn find_project_root(start: &Path) -> PathBuf {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join("Cargo.toml").exists() || current.join(".git").exists() {
            return current;
        }
        if !current.pop() {
            return start.to_path_buf();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = VibeConfig::default();
        let json = toml::to_string_pretty(&config).unwrap();
        let back: VibeConfig = toml::from_str(&json).unwrap();
        assert!(back.vibe.token_budget.is_none());
    }

    #[test]
    fn parses_full_config() {
        let toml_str = r#"
[vibe]
token_budget = 32000
tokenizer = "cl100k"
scrub = true
threshold = 7.0

[diff_risk]
enabled = true
threshold = 8.0
extra_args = ["--strict"]

[cargo_impact]
enabled = true
extra_args = ["--confidence-min", "0.7"]

[spec_drift]
enabled = false
"#;
        let config: VibeConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.vibe.token_budget, Some(32000));
        assert_eq!(config.vibe.threshold, Some(7.0));
        assert_eq!(config.diff_risk.threshold, Some(8.0));
        assert_eq!(config.cargo_impact.extra_args.len(), 2);
        assert_eq!(config.spec_drift.enabled, Some(false));
    }
}
