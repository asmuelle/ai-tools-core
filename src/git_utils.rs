use std::path::{Path, PathBuf};
use std::process::Command;

/// Run `git diff --name-status -z --find-renames` and parse the result.
pub fn changed_files(root: &Path, since: &str) -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", "-z", "--diff-filter=ACMRTUXB"])
        .arg(since)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<PathBuf> = text
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    // Add untracked .rs files too
    if let Ok(untracked) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
    {
        if untracked.status.success() {
            let untracked_text = String::from_utf8_lossy(&untracked.stdout);
            for line in untracked_text.lines() {
                if line.ends_with(".rs") {
                    let p = PathBuf::from(line);
                    if !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }
    }

    Some(files)
}

/// Get the current git diff as a unified diff string.
pub fn unified_diff(root: &Path, since: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--unified=3"])
        .arg(since)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run git blame for a specific file:line.
pub fn blame(root: &Path, file: &Path, line: u32) -> Option<crate::finding::Attribution> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["blame", "--porcelain", "-L"])
        .arg(format!("{},{}", line, line))
        .arg(file)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    parse_blame_porcelain(&text)
}

fn parse_blame_porcelain(text: &str) -> Option<crate::finding::Attribution> {
    let first_line = text.lines().next()?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 1 {
        return None;
    }
    let commit = parts[0].chars().take(7).collect::<String>();

    let mut author = String::new();
    let mut date = String::new();
    let mut summary = String::new();

    for line in text.lines() {
        if let Some(name) = line.strip_prefix("author ") {
            author = name.to_string();
        } else if let Some(ts) = line.strip_prefix("author-time ") {
            let secs: i64 = ts.parse().ok()?;
            date = unix_to_date(secs);
        } else if let Some(s) = line.strip_prefix("summary ") {
            summary = s.to_string();
        }
    }

    Some(crate::finding::Attribution {
        commit,
        author,
        date,
        summary,
    })
}

fn unix_to_date(seconds: i64) -> String {
    let days_since_epoch = seconds / 86400;
    let (y, m, d) = civil_from_days(days_since_epoch);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Check if a directory is a git repository.
pub fn is_git_repo(root: &Path) -> bool {
    root.join(".git").exists()
}
