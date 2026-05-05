use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unified severity for all tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational finding.
    Low,
    /// Review-worthy finding that usually does not block by itself.
    Medium,
    /// High-risk finding that commonly blocks strict checks.
    High,
    /// Critical finding that should always block automated approval.
    Critical,
}

impl Severity {
    /// Return the numeric risk weight used by aggregate scoring.
    pub fn weight(self) -> f32 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 3.0,
            Self::High => 6.0,
            Self::Critical => 9.0,
        }
    }

    /// Return the short display marker used by terminal renderers.
    pub fn marker(self) -> &'static str {
        match self {
            Self::Low => "🔵",
            Self::Medium => "🟡",
            Self::High => "⚠️",
            Self::Critical => "🚨",
        }
    }

    /// Return the lowercase stable label for this severity.
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
    /// Evidence is deterministic and should be treated as certain.
    Deterministic,
    /// Evidence is heuristic but strong enough to review.
    Heuristic,
    /// Evidence is experimental and may be noisy.
    Experimental,
}

impl Confidence {
    /// Return the ranking value used to compare confidence levels.
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
    /// File path for the finding.
    pub file: PathBuf,
    /// One-based line number for the finding.
    pub line: u32,
}

impl Location {
    /// Create a new source location.
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
    /// Public API or contract changed.
    ApiContract,
    /// Async boundary or executor-sensitive code changed.
    AsyncBoundary,
    /// Serialization schema changed.
    SerdeDrift,
    /// Authentication or authorization gate changed.
    AuthGate,
    /// Concurrency-sensitive code changed.
    Concurrency,
    // spec-drift / cargo-impact categories
    /// Documentation or specs mention a symbol absent from code.
    SymbolAbsence,
    /// Code violates a documented constraint.
    ConstraintViolation,
    /// Documentation or examples describe outdated behavior.
    OutdatedLogic,
    /// Example or referenced code fails to compile.
    CompileFailure,
    /// Deprecated API usage was detected.
    DeprecatedUsage,
    /// Implementation behavior appears to leave a specification gap.
    LogicGap,
    /// Test or spec claim contradicts implementation behavior.
    LyingTest,
    /// Changed behavior lacks expected test coverage.
    MissingCoverage,
    /// Documented command or workflow is not present.
    GhostCommand,
    /// Documented environment assumption is not met.
    EnvMismatch,
    // cargo-impact specific
    /// Test references a changed symbol.
    TestReference,
    /// Trait implementation is affected.
    TraitImpl,
    /// Derived trait implementation is affected.
    DerivedTraitImpl,
    /// Dynamic dispatch surface is affected.
    DynDispatch,
    /// Documentation link references changed code.
    DocDriftLink,
    /// Documentation keyword references changed code.
    DocDriftKeyword,
    /// Foreign-function interface signature changed.
    FfiSignatureChange,
    /// Build script changed.
    BuildScriptChanged,
    /// Semver-impacting change was detected.
    SemverCheck,
    /// Symbol reference was resolved.
    ResolvedReference,
    /// Runtime entry point or surface is affected.
    RuntimeSurface,
    /// Trait definition changed.
    TraitDefinitionChange,
    // Generic
    /// Rule category outside the shared taxonomy.
    Other,
}

impl RuleId {
    /// Return the stable snake_case identifier for this rule.
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
    /// Short commit hash.
    pub commit: String,
    /// Commit author.
    pub author: String,
    /// Commit date in `YYYY-MM-DD` format.
    pub date: String,
    /// Commit subject line.
    pub summary: String,
}

/// A single finding — the unified output type across all tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable finding identifier.
    pub id: String,
    /// Rule that produced the finding.
    pub rule: RuleId,
    /// Severity assigned to the finding.
    pub severity: Severity,
    /// Confidence tier assigned to the finding.
    pub confidence: Confidence,
    /// Primary source location.
    pub location: Location,
    /// Human-readable finding message.
    pub message: String,
    /// Optional remediation or next action.
    pub suggested_action: Option<String>,
    /// Optional git attribution for the source location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
    /// Tool that produced this finding.
    #[serde(default)]
    pub source_tool: String,
}

impl Finding {
    /// Create a new finding with no ID, action, attribution, or source tool.
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

    /// Attach a stable finding ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Attach a suggested remediation or next action.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }

    /// Attach the producing tool name.
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.source_tool = tool.into();
        self
    }

    /// Attach git attribution metadata.
    pub fn with_attribution(mut self, attr: Attribution) -> Self {
        self.attribution = Some(attr);
        self
    }
}

/// Compatibility type for spec-drift.
pub type Divergence = Finding;
