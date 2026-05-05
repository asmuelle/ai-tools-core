use regex::RegexSet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

/// Error returned by scrubber configuration and logging operations.
#[derive(Debug, Error)]
pub enum ScrubError {
    /// Filesystem I/O failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// YAML configuration parsing failed.
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// Regex compilation failed.
    #[error("regex: {0}")]
    Regex(#[from] regex::Error),
}

/// Configuration for the secret scrubbing pipeline.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScrubConfig {
    /// Configuration schema version.
    pub version: u32,
    /// Regex-based secret patterns.
    #[serde(default)]
    pub patterns: Vec<ScrubPattern>,
    /// Path-based redaction settings.
    #[serde(default)]
    pub paths: ScrubPathConfig,
}

/// Path-level redaction configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScrubPathConfig {
    /// Glob patterns whose matching files should be redacted as a whole.
    #[serde(default)]
    pub redact_whole: Vec<String>,
}

/// A single regex-based secret detection rule.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScrubPattern {
    /// Stable rule identifier.
    pub id: String,
    /// Regex pattern used to find the secret.
    pub regex: String,
    /// Secret category, such as `aws` or `github`.
    pub category: String,
    /// Severity label for this secret class.
    pub severity: String,
}

/// Report of what was scrubbed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrubReport {
    /// Individual redaction records.
    pub redactions: Vec<RedactionRecord>,
    /// Whether scrubbing was enabled for the operation.
    pub enabled: bool,
}

/// Metadata for one redacted secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRecord {
    /// Rule that produced the redaction.
    pub rule_id: String,
    /// Redacted secret category.
    pub category: String,
    /// Redacted secret severity.
    pub severity: String,
    /// First four hex characters of the secret hash.
    pub hash4: String,
}

impl ScrubReport {
    /// Return true when no redactions were recorded.
    pub fn is_empty(&self) -> bool {
        self.redactions.is_empty()
    }

    /// Return a short human-readable summary of redactions.
    pub fn summary(&self) -> String {
        if self.redactions.is_empty() {
            "no secrets detected".to_string()
        } else {
            let by_category: BTreeMap<&str, usize> =
                self.redactions.iter().fold(BTreeMap::new(), |mut acc, r| {
                    *acc.entry(r.category.as_str()).or_default() += 1;
                    acc
                });
            let parts: Vec<String> = by_category
                .iter()
                .map(|(cat, count)| format!("{count} {cat}"))
                .collect();
            format!(
                "{} redaction(s): {}",
                self.redactions.len(),
                parts.join(", ")
            )
        }
    }
}

/// The secret scrubbing pipeline.
pub struct Scrubber {
    pattern_set: RegexSet,
    patterns: Vec<ScrubPattern>,
    redact_paths: Vec<globset::Glob>,
    enabled: bool,
}

impl Scrubber {
    /// Create a disabled scrubber that passes input through unchanged.
    pub fn empty() -> Self {
        Self {
            pattern_set: RegexSet::empty(),
            patterns: Vec::new(),
            redact_paths: Vec::new(),
            enabled: false,
        }
    }

    /// Create a scrubber using workspace config or built-in defaults.
    pub fn with_workspace(root: &Path) -> Result<Self, ScrubError> {
        let config_path = root.join(".cargo-context").join("scrub.yaml");
        if !config_path.exists() {
            return Ok(Self::with_builtins());
        }
        let contents = std::fs::read_to_string(&config_path)?;
        let config: ScrubConfig = serde_yaml::from_str(&contents)?;
        Self::from_config(config)
    }

    /// Create a scrubber using built-in secret patterns and path rules.
    pub fn with_builtins() -> Self {
        Self::from_config(Self::builtin_config()).unwrap_or_else(|_| Self::empty())
    }

    fn builtin_config() -> ScrubConfig {
        ScrubConfig {
            version: 1,
            patterns: vec![
                ScrubPattern {
                    id: "aws_key".into(),
                    regex: r"(?i)AKIA[0-9A-Z]{16}".into(),
                    category: "aws".into(),
                    severity: "critical".into(),
                },
                ScrubPattern {
                    id: "github_token".into(),
                    regex: r"(?i)gh[pousr]_[0-9a-zA-Z]{36}".into(),
                    category: "github".into(),
                    severity: "critical".into(),
                },
                ScrubPattern {
                    id: "openai_key".into(),
                    regex: r"(?i)sk-[0-9a-zA-Z]{32,}".into(),
                    category: "openai".into(),
                    severity: "critical".into(),
                },
                ScrubPattern {
                    id: "private_key".into(),
                    regex: r"-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----".into(),
                    category: "crypto".into(),
                    severity: "critical".into(),
                },
                ScrubPattern {
                    id: "jwt".into(),
                    regex: r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}".into(),
                    category: "token".into(),
                    severity: "high".into(),
                },
            ],
            paths: ScrubPathConfig {
                redact_whole: vec!["**/.env".into(), "**/*.pem".into(), "**/*.key".into()],
            },
        }
    }

    fn from_config(config: ScrubConfig) -> Result<Self, ScrubError> {
        let patterns: Vec<String> = config.patterns.iter().map(|p| p.regex.clone()).collect();
        let pattern_set = RegexSet::new(&patterns)?;
        let redact_paths: Vec<globset::Glob> = config
            .paths
            .redact_whole
            .iter()
            .filter_map(|p| globset::Glob::new(p).ok())
            .collect();
        Ok(Self {
            pattern_set,
            patterns: config.patterns,
            redact_paths,
            enabled: true,
        })
    }

    /// Return true when this scrubber actively scans input.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Return true when a path matches a full-file redaction glob.
    pub fn is_path_redacted(&self, path: &Path) -> bool {
        self.redact_paths
            .iter()
            .any(|g| g.compile_matcher().is_match(path))
    }

    /// Scrub secrets from input and return the scrubbed text.
    pub fn scrub(&self, input: &str) -> String {
        let (scrubbed, _) = self.scrub_with_report(input);
        scrubbed
    }

    /// Scrub secrets from input and return both text and redaction report.
    pub fn scrub_with_report(&self, input: &str) -> (String, ScrubReport) {
        if !self.enabled {
            return (input.to_string(), ScrubReport::default());
        }

        let matches: Vec<_> = self.pattern_set.matches(input).into_iter().collect();
        if matches.is_empty() {
            return (input.to_string(), ScrubReport::default());
        }

        let mut report = ScrubReport {
            enabled: true,
            redactions: Vec::new(),
        };
        let mut result = input.to_string();

        for &idx in &matches {
            let pattern = &self.patterns[idx];
            let re = regex::Regex::new(&pattern.regex).unwrap();
            let mut new_matches: Vec<(usize, usize)> = Vec::new();
            for m in re.find_iter(input) {
                new_matches.push((m.start(), m.end()));
            }
            // Apply replacements from end to start to preserve indices
            new_matches.sort_by_key(|m| std::cmp::Reverse(m.0));
            for (start, end) in new_matches {
                let matched = &result[start..end];
                let hash = format!("{:x}", Sha256::digest(matched.as_bytes()));
                let hash4 = &hash[..4];
                let replacement =
                    format!("<REDACTED:{}:{}:{}>", &pattern.category, &pattern.id, hash4);
                result.replace_range(start..end, &replacement);
                report.redactions.push(RedactionRecord {
                    rule_id: pattern.id.clone(),
                    category: pattern.category.clone(),
                    severity: pattern.severity.clone(),
                    hash4: hash4.to_string(),
                });
            }
        }

        (result, report)
    }

    /// Log a short diagnostic for recorded redactions.
    pub fn log_redactions(&self, report: &ScrubReport) -> Result<(), ScrubError> {
        if report.redactions.is_empty() {
            return Ok(());
        }
        let msg = format!(
            "cargo-vibe: scrubbed {} secret(s): {}",
            report.redactions.len(),
            report.summary()
        );
        eprintln!("{msg}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scrubber_passes_through() {
        let scrubber = Scrubber::empty();
        let (result, report) = scrubber.scrub_with_report("secret: sk-abc123def456");
        assert!(result.contains("sk-abc123def456"));
        assert!(report.redactions.is_empty());
    }

    #[test]
    fn detects_openai_key() {
        let scrubber = Scrubber::with_builtins();
        let (result, report) =
            scrubber.scrub_with_report("apikey=sk-proj1234567890abcdefghijklmnopqrstuvwxyz");
        assert!(result.contains("<REDACTED:openai:openai_key:"));
        assert!(!report.redactions.is_empty());
    }

    #[test]
    fn detects_aws_key() {
        let scrubber = Scrubber::with_builtins();
        let (result, report) = scrubber.scrub_with_report("AWS_ACCESS_KEY=AKIA1234567890ABCDEF");
        assert!(result.contains("<REDACTED:aws:"));
        assert!(!report.redactions.is_empty());
    }

    #[test]
    fn detects_jwt() {
        let scrubber = Scrubber::with_builtins();
        let (result, report) = scrubber.scrub_with_report(
            "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUPFK9HqJA",
        );
        assert!(result.contains("<REDACTED:token:"));
        assert!(!report.redactions.is_empty());
    }

    #[test]
    fn redacts_pem_files_by_path() {
        let scrubber = Scrubber::with_builtins();
        assert!(scrubber.is_path_redacted(Path::new("secrets/private.pem")));
        assert!(scrubber.is_path_redacted(Path::new(".env")));
        assert!(!scrubber.is_path_redacted(Path::new("src/lib.rs")));
    }

    #[test]
    fn scrub_with_report_handles_no_matches() {
        let scrubber = Scrubber::with_builtins();
        let (result, report) = scrubber.scrub_with_report("safe content");
        assert_eq!(result, "safe content");
        assert!(report.redactions.is_empty());
    }
}
