//! Shared foundation types and utilities for Rust AI coding tools.

/// Cargo metadata and workspace helper utilities.
pub mod cargo_utils;
/// Unified project configuration types and discovery helpers.
pub mod config;
/// Unified finding, severity, confidence, rule, and location types.
pub mod finding;
/// Git command helper utilities.
pub mod git_utils;
/// SARIF v2.1.0 rendering support.
pub mod sarif;
/// Secret scrubbing configuration and runtime support.
pub mod scrub;

pub use config::{VibeConfig, VibeToolConfig, load_project_config};
pub use finding::{
    Confidence, Divergence, Finding, Location, Location as CommonLocation, RuleId, Severity,
    SeverityClass,
};
pub use sarif::SarifRenderer;
pub use scrub::{ScrubConfig, ScrubReport, Scrubber};
