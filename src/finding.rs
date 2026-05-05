use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unified severity for all tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn weight(self) -> f32 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 3.0,
            Self::High => 6.0,
            Self::Critical => 9.0,
        }
    }

    pub fn marker(self) -> &'static str {
        match self {
            Self::Low => "🔵",
            Self::Medium => "🟡",
            Self::High => "⚠️",
            Self::Critical => "🚨",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Compatibility alias for tools using SeverityClass.
pub type SeverityClass = Severity;

/// Confidence tier for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Deterministic,
    Heuristic,
    Experimental,
}

impl Confidence {
    pub fn rank(self) -> u8 {
        match self {
            Self::Deterministic => 3,
            Self::Heuristic => 2,
            Self::Experimental => 1,
        }
    }
}

/// Source location: file + line.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
}

impl Location {
    pub fn new(file: impl Into<PathBuf>, line: u32) -> Self {
        Self {
            file: file.into(),
            line,
        }
    }
}

/// Categorization for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleId {
    // diff-risk categories
    ApiContract,
    AsyncBoundary,
    SerdeDrift,
    AuthGate,
    Concurrency,
    // spec-drift / cargo-impact categories
    SymbolAbsence,
    ConstraintViolation,
    OutdatedLogic,
    CompileFailure,
    DeprecatedUsage,
    LogicGap,
    LyingTest,
    MissingCoverage,
    GhostCommand,
    EnvMismatch,
    // cargo-impact specific
    TestReference,
    TraitImpl,
    DerivedTraitImpl,
    DynDispatch,
    DocDriftLink,
    DocDriftKeyword,
    FfiSignatureChange,
    BuildScriptChanged,
    SemverCheck,
    ResolvedReference,
    RuntimeSurface,
    TraitDefinitionChange,
    // Generic
    Other,
}

impl RuleId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiContract => "api_contract",
            Self::AsyncBoundary => "async_boundary",
            Self::SerdeDrift => "serde_drift",
            Self::AuthGate => "auth_gate",
            Self::Concurrency => "concurrency",
            Self::SymbolAbsence => "symbol_absence",
            Self::ConstraintViolation => "constraint_violation",
            Self::OutdatedLogic => "outdated_logic",
            Self::CompileFailure => "compile_failure",
            Self::DeprecatedUsage => "deprecated_usage",
            Self::LogicGap => "logic_gap",
            Self::LyingTest => "lying_test",
            Self::MissingCoverage => "missing_coverage",
            Self::GhostCommand => "ghost_command",
            Self::EnvMismatch => "env_mismatch",
            Self::TestReference => "test_reference",
            Self::TraitImpl => "trait_impl",
            Self::DerivedTraitImpl => "derived_trait_impl",
            Self::DynDispatch => "dyn_dispatch",
            Self::DocDriftLink => "doc_drift_link",
            Self::DocDriftKeyword => "doc_drift_keyword",
            Self::FfiSignatureChange => "ffi_signature_change",
            Self::BuildScriptChanged => "build_script_changed",
            Self::SemverCheck => "semver_check",
            Self::ResolvedReference => "resolved_reference",
            Self::RuntimeSurface => "runtime_surface",
            Self::TraitDefinitionChange => "trait_definition_change",
            Self::Other => "other",
        }
    }
}

/// Git blame attribution for a finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attribution {
    pub commit: String,
    pub author: String,
    pub date: String,
    pub summary: String,
}

/// A single finding — the unified output type across all tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub rule: RuleId,
    pub severity: Severity,
    pub confidence: Confidence,
    pub location: Location,
    pub message: String,
    pub suggested_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
    /// Tool that produced this finding.
    #[serde(default)]
    pub source_tool: String,
}

impl Finding {
    pub fn new(
        rule: RuleId,
        severity: Severity,
        confidence: Confidence,
        location: Location,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: String::new(),
            rule,
            severity,
            confidence,
            location,
            message: message.into(),
            suggested_action: None,
            attribution: None,
            source_tool: String::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.source_tool = tool.into();
        self
    }

    pub fn with_attribution(mut self, attr: Attribution) -> Self {
        self.attribution = Some(attr);
        self
    }
}

/// Compatibility type for spec-drift.
pub type Divergence = Finding;
