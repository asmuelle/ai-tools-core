use crate::finding::Finding;
use serde::Serialize;
use std::collections::BTreeMap;

/// SARIF v2.1.0 renderer.
///
/// Produces output compatible with GitHub Code Scanning, GitLab SAST, and
/// other SARIF-consuming tools. Uses `partialFingerprints` for deduplication.
#[derive(Default)]
pub struct SarifRenderer {
    tool_name: String,
    tool_version: String,
    tool_info_uri: String,
}

impl SarifRenderer {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            tool_name: name.to_string(),
            tool_version: version.to_string(),
            tool_info_uri: String::new(),
        }
    }

    pub fn with_info_uri(mut self, uri: &str) -> Self {
        self.tool_info_uri = uri.to_string();
        self
    }

    pub fn render(&self, findings: &[Finding]) -> String {
        let mut rules: BTreeMap<String, SarifRule> = BTreeMap::new();
        for f in findings {
            let rule_id = f.rule.as_str().to_string();
            rules.entry(rule_id.clone()).or_insert_with(|| SarifRule {
                id: rule_id.clone(),
                name: rule_id.clone(),
                short_description: SarifMessage {
                    text: rule_id.clone(),
                },
                full_description: SarifMessage {
                    text: format!("Finding from rule {}", rule_id),
                },
                help_uri: String::new(),
            });
        }

        let mut results: Vec<SarifResult> = Vec::new();
        for f in findings {
            let rule_id = f.rule.as_str().to_string();
            let mut partial_fingerprints = BTreeMap::new();
            partial_fingerprints.insert(
                "primaryLocationLineHash".to_string(),
                f.id.clone(),
            );

            results.push(SarifResult {
                rule_id: Some(rule_id),
                rule_index: None,
                level: match f.severity {
                    crate::finding::Severity::Low => "note",
                    crate::finding::Severity::Medium => "warning",
                    crate::finding::Severity::High => "error",
                    crate::finding::Severity::Critical => "error",
                }
                .to_string(),
                message: SarifMessage {
                    text: f.message.clone(),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: f.location.file.to_string_lossy().to_string(),
                        },
                        region: SarifRegion {
                            start_line: f.location.line,
                            start_column: Some(1),
                        },
                    },
                }],
                partial_fingerprints,
            });
        }

        let sarif = SarifLog {
            version: "2.1.0".to_string(),
            schema: String::new(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifToolComponent {
                        name: self.tool_name.clone(),
                        version: self.tool_version.clone(),
                        information_uri: self.tool_info_uri.clone(),
                        rules: rules.into_values().collect(),
                    },
                },
                results,
            }],
        };

        serde_json::to_string_pretty(&sarif).unwrap_or_default()
    }
}

#[derive(Serialize)]
struct SarifLog {
    version: String,
    #[serde(rename = "$schema", skip_serializing_if = "String::is_empty")]
    schema: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifToolComponent,
}

#[derive(Serialize)]
struct SarifToolComponent {
    name: String,
    version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    information_uri: String,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    name: String,
    short_description: SarifMessage,
    full_description: SarifMessage,
    #[serde(skip_serializing_if = "String::is_empty")]
    help_uri: String,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_index: Option<usize>,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    partial_fingerprints: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Confidence, Location, RuleId, Severity};

    #[test]
    fn renders_valid_sarif() {
        let findings = vec![Finding::new(
            RuleId::ApiContract,
            Severity::High,
            Confidence::Deterministic,
            Location::new("src/lib.rs", 42),
            "Public API changed",
        )
        .with_id("f-001")
        .with_tool("diff-risk")];

        let renderer = SarifRenderer::new("diff-risk", "0.2.0");
        let output = renderer.render(&findings);

        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("diff-risk"));
        assert!(output.contains("f-001"));
        // Valid JSON
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn empty_findings_produces_valid_sarif() {
        let renderer = SarifRenderer::new("test", "0.1.0");
        let output = renderer.render(&[]);
        assert!(output.contains("\"results\": []"));
    }
}
