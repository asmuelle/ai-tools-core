# ai-tools-core

[![CI](https://github.com/asmuelle/ai-tools-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/asmuelle/ai-tools-core/actions/workflows/ci.yml)

Shared foundation crate for the Rust AI vibe coding toolchain. Provides unified types, utilities, and configuration shared by `cargo-context`, `diff-risk`, `cargo-impact`, `spec-drift`, and `cargo-vibe`.

## Why

All four tools in the toolchain reimplement the same primitives — findings, severity, location, SARIF rendering, git operations, cargo metadata parsing, secret scrubbing. `ai-tools-core` consolidates them into one dependency, ensuring consistent output formats, canonical type definitions, and a single source of truth for the `.cargo-vibe.toml` config schema.

## Modules

| Module | Purpose |
|--------|---------|
| `finding` | Unified `Finding`, `Severity` (Low/Medium/High/Critical), `Confidence` (Deterministic/Heuristic/Experimental), `Location`, `RuleId` (27 variants covering all four tools), `Attribution` |
| `sarif` | SARIF v2.1.0 renderer. Produces GitHub Code Scanning / GitLab SAST compatible output from any `Finding` slice. |
| `git_utils` | `changed_files()` (git diff name-only), `unified_diff()`, `blame()` (porcelain parser with Howard Hinnant civil date algorithm, zero `chrono` dependency), `is_git_repo()` |
| `cargo_utils` | `cargo metadata` parser (`CargoMetadata`, `Package`, `Target`), `workspace_root()`, `find_rust_files()`, `is_rust_file()` |
| `scrub` | Three-layer secret scrubbing pipeline: regex patterns (AWS keys, GitHub tokens, OpenAI keys, JWT, private keys), Shannon entropy detection, path-based file redaction. Configurable via `.cargo-context/scrub.yaml`. |
| `config` | `.cargo-vibe.toml` parser with `VibeGlobalConfig` and per-tool `VibeToolConfig` sections. `load_project_config()` walks up from `start_dir` to find the config file. |

## Types

### Finding

The canonical output type across all tools:

```rust
pub struct Finding {
    pub id: String,              // stable content-hashed ID
    pub rule: RuleId,            // which rule produced this
    pub severity: Severity,      // Low | Medium | High | Critical
    pub confidence: Confidence,  // Deterministic | Heuristic | Experimental
    pub location: Location,      // file + line
    pub message: String,         // human-readable explanation
    pub suggested_action: Option<String>,
    pub attribution: Option<Attribution>,  // git blame
    pub source_tool: String,     // "diff-risk" | "cargo-impact" | "spec-drift"
}
```

### Severity

| Value | Weight | Marker | CI Behavior |
|-------|--------|--------|-------------|
| `Low` | 1.0 | 🔵 | Informational only |
| `Medium` | 3.0 | 🟡 | Requires review |
| `High` | 6.0 | ⚠️ | Blocks PR (with `--fail-on high`) |
| `Critical` | 9.0 | 🚨 | Always blocks |

### Confidence

| Tier | Meaning | Example |
|------|---------|---------|
| `Deterministic` | Tool is certain | `cargo check` compile error, `diff-risk` regex match |
| `Heuristic` | Strong signal but not proven | `cargo-impact` syn-based test reference, `spec-drift` lying test |
| `Experimental` | LLM-backed, may hallucinate | `spec-drift` outdated logic, logic gap |

### RuleId

```rust
// diff-risk
ApiContract | AsyncBoundary | SerdeDrift | AuthGate | Concurrency

// spec-drift
SymbolAbsence | ConstraintViolation | CompileFailure | DeprecatedUsage
| LyingTest | MissingCoverage | GhostCommand | EnvMismatch
| OutdatedLogic | LogicGap

// cargo-impact
TestReference | TraitImpl | DerivedTraitImpl | DynDispatch
| DocDriftLink | DocDriftKeyword | FfiSignatureChange | BuildScriptChanged
| SemverCheck | ResolvedReference | RuntimeSurface | TraitDefinitionChange

// Generic
Other
```

## Configuration

### `.cargo-vibe.toml`

```toml
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
enabled = true

[cargo_context]
enabled = true
```

Config discovery walks upward from the current directory, checking for `.cargo-vibe.toml`. Tools that have their own config files (`.cargo-context/config.yaml`, `cargo-impact.toml`, `spec-drift.toml`) continue to work; `.cargo-vibe.toml` provides overrides and a unified control surface.

## Secret Scrubbing

The three-layer pipeline runs in sequence:

1. **Pattern matching** — `RegexSet` of 5 built-in patterns (AWS keys, GitHub tokens, OpenAI keys, PEM private keys, JWTs). Extensible via `.cargo-context/scrub.yaml`.
2. **Entropy detection** — Shannon entropy > 4.5 on tokens ≥ 20 chars near suspicious key names.
3. **Path redaction** — Glob patterns for files that should never be included (`.env`, `*.pem`, `*.key`).

Redacted content is replaced with `<REDACTED:category:rule_id:hash4>` format. A `ScrubReport` tracks every redaction with rule ID, category, severity, and partial hash.

## Usage

Add to your tool's `Cargo.toml`:

```toml
[dependencies]
ai-tools-core = { path = "../ai-tools-core" }
```

```rust
use ai_tools_core::{
    finding::{Finding, Severity, Confidence, Location, RuleId},
    sarif::SarifRenderer,
    git_utils,
    scrub::Scrubber,
    config::load_project_config,
};

// Build a finding
let finding = Finding::new(
    RuleId::ApiContract,
    Severity::High,
    Confidence::Deterministic,
    Location::new("src/lib.rs", 42),
    "Public API changed: fn old() -> fn new()",
)
.with_id("f-a1b2c3d4")
.with_tool("diff-risk")
.with_action("Verify all callers are updated");

// Render as SARIF
let renderer = SarifRenderer::new("my-tool", "0.1.0");
println!("{}", renderer.render(&[finding]));

// Load project config
if let Some(config) = load_project_config(&std::env::current_dir().unwrap()) {
    println!("Token budget: {:?}", config.vibe.token_budget);
}

// Scrub secrets from output
let scrubber = Scrubber::with_builtins();
let (safe, report) = scrubber.scrub_with_report("api_key=sk-abc123...");
assert!(safe.contains("<REDACTED:openai:openai_key:"));
```

## Dependencies

Minimal and carefully chosen:

| Crate | Why |
|-------|-----|
| `serde` / `serde_json` | Type serialization, SARIF output |
| `serde_yaml` | `.cargo-context/scrub.yaml` parsing |
| `toml` | `.cargo-vibe.toml` config |
| `regex` | Secret pattern matching |
| `globset` | Path-based redaction and include/exclude |
| `sha2` | Content hashing for stable IDs and cache keys |
| `thiserror` | Error types |
| `log` | Diagnostic logging |

## License

MIT OR Apache-2.0
