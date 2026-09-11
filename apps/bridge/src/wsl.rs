//! WSL2 distro discovery helpers.
//!
//! On Windows: prefer `wsl.exe -l -v` (and optionally registry). Access Linux homes via
//! `\\wsl$\<Distro>\home\<user>\...` **or** `wsl -d <Distro> -- ...` (both documented).
//! On Linux (this POC box): use injectable [`crate::types::WslDistroProbe`] fixtures.

use crate::errors::{make_error, TT_F2_001, TT_F2_002, TT_F2_003};
use crate::types::{DiscoverError, WslDistroProbe};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of listing WSL distros (live or fixture).
#[derive(Debug, Clone)]
pub struct WslListResult {
    pub distros: Vec<WslDistroProbe>,
    pub errors: Vec<DiscoverError>,
}

/// Decode `wsl.exe` stdout which is often UTF-16LE (with or without BOM).
pub fn decode_wsl_output(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.iter().any(|&b| b == 0) {
        // Strip BOM if present, then take low bytes of UTF-16LE pairs.
        let raw = if bytes.starts_with(&[0xFF, 0xFE]) {
            &bytes[2..]
        } else {
            bytes
        };
        String::from_utf8_lossy(&raw.chunks(2).map(|c| c[0]).collect::<Vec<u8>>()).into_owned()
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
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

/// Parse whoami/home probe output: two non-empty lines = USER then HOME.
/// Accepts `printf '%s\n%s\n' "$USER" "$HOME"` or `id -un` + `echo $HOME`.
pub fn parse_wsl_whoami_home(output: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 2 {
        return None;
    }
    let user = lines[0].to_string();
    let home = lines[1].to_string();
    if user.is_empty() || home.is_empty() || user.contains('/') || user.contains('\\') {
        return None;
    }
    // Prefer HOME that looks like an absolute Linux path; still accept if USER is sane.
    if !home.starts_with('/') {
        return None;
    }
    Some((user, home))
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

/// Resolve default WSL user + home for one distro via `wsl.exe -d <Distro> -- bash -lc ...`.
/// Tries even when the distro is Stopped (WSL may auto-start); on failure returns TT-F2-002/003.
pub fn resolve_wsl_user_home(distro: &str) -> Result<(String, PathBuf), DiscoverError> {
    let script = r#"printf '%s\n%s\n' "$USER" "$HOME""#;
    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "bash", "-lc", script])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let raw = decode_wsl_output(&out.stdout);
            match parse_wsl_whoami_home(&raw) {
                Some((user, _linux_home)) => {
                    let unc = PathBuf::from(wsl_unc_home(distro, &user));
                    Ok((user, unc))
                }
                None => Err(make_error(
                    TT_F2_002,
                    format!(
                        "WSL distro '{distro}': could not parse USER/HOME from probe output: {:?}",
                        raw.trim()
                    ),
                    Some(format!(r"\\wsl$\{distro}")),
                )),
            }
        }
        Ok(out) => {
            let stderr = decode_wsl_output(&out.stderr);
            let stdout = decode_wsl_output(&out.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };
            Err(make_error(
                TT_F2_003,
                format!(
                    "WSL distro '{distro}': failed to resolve user/home (exit {:?}): {detail}",
                    out.status.code()
                ),
                Some(format!(r"\\wsl$\{distro}")),
            ))
        }
        Err(e) => Err(make_error(
            TT_F2_002,
            format!("WSL distro '{distro}': failed to invoke wsl.exe for user/home: {e}"),
            Some(format!(r"\\wsl$\{distro}")),
        )),
    }
}

/// Attempt live `wsl.exe -l -v` on Windows, then resolve each distro's user + `\\wsl$\...\home\<user>`.
/// On non-Windows returns empty (callers use injected fixtures).
/// Failed per-distro resolution emits TT-F2-002/003 and skips that distro (never invents `unknown`).
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
            let raw = decode_wsl_output(&out.stdout);
            let mut distros = Vec::new();
            let mut errors = Vec::new();
            for (name, state, version) in parse_wsl_list_v(&raw) {
                // Stopped distros: still try (WSL often auto-starts); emit clear diagnostic on failure.
                match resolve_wsl_user_home(&name) {
                    Ok((user, linux_home)) => {
                        distros.push(WslDistroProbe {
                            name,
                            state,
                            version: Some(version),
                            wsl_user: Some(user),
                            linux_home,
                        });
                    }
                    Err(e) => {
                        let mut err = e;
                        err.message = format!(
                            "{} (distro state was '{state}')",
                            err.message
                        );
                        errors.push(err);
                    }
                }
            }
            WslListResult { distros, errors }
        }
        Ok(out) => {
            let stderr = decode_wsl_output(&out.stderr);
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

    #[test]
    fn parse_whoami_home_printf_style() {
        let out = "t3261\n/home/t3261\n";
        let (u, h) = parse_wsl_whoami_home(out).expect("parse");
        assert_eq!(u, "t3261");
        assert_eq!(h, "/home/t3261");
    }

    #[test]
    fn parse_whoami_home_rejects_garbage() {
        assert!(parse_wsl_whoami_home("").is_none());
        assert!(parse_wsl_whoami_home("onlyone\n").is_none());
        assert!(parse_wsl_whoami_home("user\nrelative\n").is_none());
        assert!(parse_wsl_whoami_home("bad/user\n/home/x\n").is_none());
    }

    #[test]
    fn decode_utf16le_strips_nuls() {
        // "Hi\n" as UTF-16LE without BOM
        let bytes = [b'H', 0, b'i', 0, b'\n', 0];
        let s = decode_wsl_output(&bytes);
        assert_eq!(s.trim(), "Hi");
    }
}
