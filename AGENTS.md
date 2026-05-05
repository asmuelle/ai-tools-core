# AGENTS.md

Orientation for AI assistants working in this repo. Loaded automatically
by Codex; readable by other agents that respect this convention.

## What this crate is

`ai-tools-core` is the shared foundation library for the Rust AI vibe coding
toolchain. It provides unified types (`Finding`, `Severity`, `Confidence`,
`Location`, `RuleId`, `Attribution`), a SARIF v2.1.0 renderer, git/cargo
utilities, a three-layer secret scrubbing pipeline, and the `.cargo-vibe.toml`
config parser. Every other tool in the family (`cargo-context`, `diff-risk`,
`cargo-impact`, `spec-drift`, `cargo-vibe`) depends on it — directly or
conceptually.

## Library-first, I/O-free where possible

This crate is a library, not a binary. It has no async runtime, no terminal
I/O, and no process spawning. All git and cargo operations are pure function
calls that shell out internally, exposing regular `Option`/`Result` returns.

## Module map

| Module | Role |
|--------|------|
| `finding.rs` | `Finding`, `Severity` (Low/Medium/High/Critical), `Confidence` (Deterministic/Heuristic/Experimental), `Location`, `RuleId` (27 variants), `Attribution` |
| `sarif.rs` | `SarifRenderer` — produces GitHub Code Scanning / GitLab SAST output |
| `git_utils.rs` | `changed_files()`, `unified_diff()`, `blame()`, `is_git_repo()` — Howard Hinnant civil date algorithm (zero `chrono` dep) |
| `cargo_utils.rs` | `load_metadata()` (`CargoMetadata`/`Package`/`Target`), `workspace_root()`, `find_rust_files()` |
| `scrub.rs` | `Scrubber` — pattern + entropy + path-glob pipeline, `ScrubReport`, `ScrubConfig` (from `.cargo-context/scrub.yaml`) |
| `config.rs` | `VibeConfig` / `VibeToolConfig` — parses `.cargo-vibe.toml` with `load_project_config()` upward walk |

## Design invariants

- **No `Deserialize` on `Finding` that depends on external crates.** Serialization
  is one-way; tools produce Findings, consumers render them. Round-trip only
  through SARIF or JSON envelope formats.
- **`RuleId` is the canonical rule namespace.** Before adding a variant, check
  whether an existing one covers the category. `Other` exists as an escape hatch
  but should be rare.
- **`Severity` weights are stable.** 1/3/6/9. The `diff-risk` scoring formula
  depends on these. Changing them requires recalibrating the calibration corpus.
- **`Confidence` is a three-value enum.** Don't add fractional confidence here;
  that's what the `confidence` field on `Finding` is for (0.0–1.0 float).
- **Secret scrubber is fail-closed.** If `scrub.yaml` is malformed, fall back
  to built-in patterns rather than disabling scrubbing entirely.
- **Git operations degrade gracefully.** If git is not available, functions
  return `None` — never panic.

## Local-verify triple before any commit

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Commit style

Follow the existing tool convention: conventional-ish commits that lead with
the *why*, name the affected modules, and call out deferred scope explicitly.
Short `fix:` / `chore:` one-liners are fine for tiny changes.

## Honest caveats

- **`blame()` date conversion uses a custom civil-from-days algorithm** —
  handles dates from year 0 onward, but edge cases around leap seconds are
  not handled. For attribution purposes this is sufficient.
- **Entropy-based scrubbing is a heuristic.** Tokens ≥ 20 chars with Shannon
  entropy > 4.5 near suspicious key names are flagged. False positives are
  possible on base64-encoded binary blobs.
- **`cargo_utils::load_metadata()` shells out** — it spawns `cargo metadata`
  as a subprocess. Not I/O-free. Callers should cache the result.
- **`RuleId::Other` is a catch-all.** Tools should prefer specific variants.
  If you find yourself using `Other` frequently, add a new variant.

## Where to file things

- Bugs / feature requests: https://github.com/asmuelle/ai-tools/issues
- Interop questions: same repo; this crate is the integration surface between
  the other four tools.

## Don't commit without

- `fmt` + `clippy -D warnings` + `test` all green
- Updated the module map in this file if you add or rename a module
- A commit message that names the modules/files touched
