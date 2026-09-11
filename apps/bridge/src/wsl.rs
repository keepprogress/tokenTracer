//! WSL2 distro discovery helpers.
//!
//! On Windows: prefer `wsl.exe -l -v` (and optionally registry). Access Linux homes via
//! `\\wsl$\<Distro>\home\<user>\...` **or** `wsl -d <Distro> -- ...` (both documented).
//! On Linux (this POC box): use injectable [`crate::types::WslDistroProbe`] fixtures.

use crate::errors::{make_error, TT_F2_001, TT_F2_002};
use crate::types::{DiscoverError, WslDistroProbe};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of listing WSL distros (live or fixture).
#[derive(Debug, Clone)]
pub struct WslListResult {
    pub distros: Vec<WslDistroProbe>,
    pub errors: Vec<DiscoverError>,
}

/// Parse `wsl.exe -l -v` style UTF-16/UTF-8 text into name/state/version triples.
/// Tolerant of a header line and `*` default marker.
pub fn parse_wsl_list_v(output: &str) -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("name") && lower.contains("state") {
            continue;
        }
        let cleaned = trimmed.trim_start_matches('*').trim();
        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0].to_string();
        let state = parts[1].to_string();
        let version = parts.get(2).copied().unwrap_or("2").to_string();
        rows.push((name, state, version));
    }
    rows
}

/// Build UNC-style path string for documentation / Windows importer.
pub fn wsl_unc_home(distro: &str, user: &str) -> String {
    format!(r"\\wsl$\{distro}\home\{user}")
}

/// Documented alternate access: `wsl -d Distro -- ...`
pub fn wsl_exec_prefix(distro: &str) -> String {
    format!("wsl -d {distro} --")
}

/// Resolve Linux home under an injectable mount (fixture or `\\wsl$\...` mapped path).
pub fn join_linux_home(home: &Path, rel: &str) -> PathBuf {
    home.join(rel)
}

/// Attempt live `wsl.exe -l -v` on Windows. On non-Windows returns empty + optional error
/// when the caller expected WSL without injection.
pub fn try_live_wsl_list() -> WslListResult {
    if !cfg!(target_os = "windows") {
        return WslListResult {
            distros: Vec::new(),
            errors: Vec::new(),
        };
    }

    let output = Command::new("wsl.exe").args(["-l", "-v"]).output();
    match output {
        Ok(out) if out.status.success() => {
            // wsl.exe often emits UTF-16LE; try lossy UTF-8 first, then strip NULs.
            let raw = if out.stdout.starts_with(&[0xFF, 0xFE]) || out.stdout.iter().any(|&b| b == 0) {
                String::from_utf8_lossy(
                    &out
                        .stdout
                        .chunks(2)
                        .map(|c| c[0])
                        .collect::<Vec<u8>>(),
                )
                .into_owned()
            } else {
                String::from_utf8_lossy(&out.stdout).into_owned()
            };
            let mut distros = Vec::new();
            for (name, state, version) in parse_wsl_list_v(&raw) {
                // Live probe leaves linux_home empty until user/home resolution (bridge follow-up).
                distros.push(WslDistroProbe {
                    name: name.clone(),
                    state,
                    version: Some(version),
                    wsl_user: None,
                    linux_home: PathBuf::from(format!(r"\\wsl$\{name}")),
                });
            }
            WslListResult {
                distros,
                errors: Vec::new(),
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            WslListResult {
                distros: Vec::new(),
                errors: vec![make_error(
                    TT_F2_002,
                    format!("wsl.exe -l -v failed: {stderr}"),
                    None,
                )],
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") || msg.contains("cannot find") {
                WslListResult {
                    distros: Vec::new(),
                    errors: vec![make_error(
                        TT_F2_001,
                        "WSL not installed or wsl.exe not on PATH",
                        None,
                    )],
                }
            } else {
                WslListResult {
                    distros: Vec::new(),
                    errors: vec![make_error(
                        TT_F2_002,
                        format!("Failed to invoke wsl.exe: {msg}"),
                        None,
                    )],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wsl_list_v_sample() {
        let sample = r#"
  NAME            STATE           VERSION
* Ubuntu          Running         2
  Debian          Stopped         2
"#;
        let rows = parse_wsl_list_v(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "Ubuntu");
        assert_eq!(rows[0].1, "Running");
        assert_eq!(rows[1].0, "Debian");
    }

    #[test]
    fn unc_and_exec_helpers() {
        assert_eq!(
            wsl_unc_home("Ubuntu", "t3261"),
            r"\\wsl$\Ubuntu\home\t3261"
        );
        assert!(wsl_exec_prefix("Ubuntu").contains("wsl -d Ubuntu"));
    }
}
