use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;

/// Parsed cargo metadata for workspace analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct CargoMetadata {
    pub workspace_root: PathBuf,
    #[serde(default)]
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub manifest_path: PathBuf,
    #[serde(default)]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub name: String,
    #[serde(default)]
    pub kind: Vec<String>,
}

/// Load cargo metadata for the workspace rooted at `root`.
pub fn load_metadata(root: &Path) -> Option<CargoMetadata> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .current_dir(root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

/// Check if cargo is available.
pub fn cargo_available() -> bool {
    Command::new("cargo").arg("--version").output().is_ok()
}

/// Get workspace root from a given directory.
pub fn workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() {
            // Check if it has a [workspace] section
            if let Ok(contents) = std::fs::read_to_string(current.join("Cargo.toml")) {
                if contents.contains("[workspace]") {
                    return Some(current);
                }
            }
            // Even if not a workspace root, return it as the package root
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Check if a path is a Rust file.
pub fn is_rust_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
}

/// Find all .rs files in a directory (excluding target/ and .git/).
pub fn find_rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                files.extend(find_rust_files(&path));
            } else if is_rust_file(&path) {
                files.push(path);
            }
        }
    }
    files
}
