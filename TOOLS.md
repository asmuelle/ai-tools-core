# TOOLS.md

Tool catalog for AI assistants operating in this repo. Lists every
available surface — binaries, library entry points, and test commands —
so agents can make tool calls without guessing at `--help` output.

## Binary (none)

`ai-tools-core` is a library crate with no binary target. It is consumed
by other tools via `Cargo.toml` path dependency:

```toml
[dependencies]
ai-tools-core = { path = "../ai-tools-core" }
```

## Library entry points

| Function / Type | Module | Signature |
|-----------------|--------|-----------|
| `Finding::new()` | `finding` | `(RuleId, Severity, Confidence, Location, impl Into<String>) -> Finding` |
| `Finding::with_id()` / `with_action()` / `with_tool()` / `with_attribution()` | `finding` | Builder methods |
| `SarifRenderer::new()` | `sarif` | `(name: &str, version: &str) -> Self` |
| `SarifRenderer::render()` | `sarif` | `(&self, &[Finding]) -> String` |
| `Scrubber::with_builtins()` | `scrub` | `() -> Self` |
| `Scrubber::with_workspace()` | `scrub` | `(root: &Path) -> Result<Self, ScrubError>` |
| `Scrubber::scrub()` / `scrub_with_report()` | `scrub` | `(&self, &str) -> (String, ScrubReport)` |
| `Scrubber::is_path_redacted()` | `scrub` | `(&self, &Path) -> bool` |
| `config::load_project_config()` | `config` | `(start_dir: &Path) -> Option<VibeConfig>` |
| `config::find_project_root()` | `config` | `(start: &Path) -> PathBuf` |
| `git_utils::changed_files()` | `git_utils` | `(root: &Path, since: &str) -> Option<Vec<PathBuf>>` |
| `git_utils::unified_diff()` | `git_utils` | `(root: &Path, since: &str) -> Option<String>` |
| `git_utils::blame()` | `git_utils` | `(root: &Path, file: &Path, line: u32) -> Option<Attribution>` |
| `git_utils::is_git_repo()` | `git_utils` | `(root: &Path) -> bool` |
| `cargo_utils::load_metadata()` | `cargo_utils` | `(root: &Path) -> Option<CargoMetadata>` |
| `cargo_utils::workspace_root()` | `cargo_utils` | `(start: &Path) -> Option<PathBuf>` |
| `cargo_utils::find_rust_files()` | `cargo_utils` | `(root: &Path) -> Vec<PathBuf>` |

## Key types (for deserialization)

| Type | Module | Fields |
|------|--------|--------|
| `Finding` | `finding` | `id`, `rule` (RuleId), `severity` (Severity), `confidence` (Confidence), `location` (Location), `message`, `suggested_action`, `attribution`, `source_tool` |
| `Location` | `finding` | `file: PathBuf`, `line: u32` |
| `Attribution` | `finding` | `commit: String`, `author: String`, `date: String`, `summary: String` |
| `VibeConfig` | `config` | `vibe: VibeGlobalConfig`, `diff_risk: VibeToolConfig`, `cargo_impact: VibeToolConfig`, `spec_drift: VibeToolConfig`, `cargo_context: VibeToolConfig` |
| `VibeGlobalConfig` | `config` | `token_budget: Option<usize>`, `tokenizer: Option<String>`, `scrub: Option<bool>`, `threshold: Option<f32>` |
| `VibeToolConfig` | `config` | `enabled: Option<bool>`, `threshold: Option<f32>`, `extra_args: Vec<String>` |
| `CargoMetadata` | `cargo_utils` | `workspace_root: PathBuf`, `packages: Vec<Package>` |
| `Package` | `cargo_utils` | `name: String`, `manifest_path: PathBuf`, `targets: Vec<Target>` |
| `Target` | `cargo_utils` | `name: String`, `kind: Vec<String>` |

## Test commands

```bash
# Run all tests
cargo test --all-features

# Run specific module tests
cargo test --lib scrub
cargo test --lib sarif
cargo test --lib config
cargo test --lib finding

# Run with output
cargo test --all-features -- --nocapture

# Check compilation only (faster)
cargo check --all-features
```

## Build commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check with all features
cargo check --all-features

# Documentation
cargo doc --no-deps --all-features --open
```
