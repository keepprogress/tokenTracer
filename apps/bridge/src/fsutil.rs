//! Filesystem helpers for discovery (read-only).

use crate::errors::{make_error, permission_denied_code, TT_F2_003, TT_F2_005};
use crate::types::{DiscoverError, SourceHost, SourceStatus};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct GlobMatch {
    pub files: Vec<String>,
    pub file_count: u64,
    /// True when `cap > 0` and more matches existed than returned in `files`.
    pub truncated: bool,
    pub readable: bool,
    pub status: SourceStatus,
    pub error: Option<DiscoverError>,
}

/// Match files under `root` whose relative path matches a simple glob.
///
/// Defaults host to Windows for callers that do not thread a host (Win/WSL
/// PermissionDenied stays TT-F2-004). Prefer [`probe_glob_for_host`] from discover.
pub fn probe_glob(root: &Path, glob: &str, cap: usize) -> GlobMatch {
    probe_glob_for_host(root, glob, cap, SourceHost::Windows)
}

/// Like [`probe_glob`], but maps PermissionDenied to TT-F10-FDA on macOS hosts.
pub fn probe_glob_for_host(root: &Path, glob: &str, cap: usize, host: SourceHost) -> GlobMatch {
    if !root.exists() {
        return GlobMatch {
            files: Vec::new(),
            file_count: 0,
            truncated: false,
            readable: false,
            status: SourceStatus::Error,
            error: Some(make_error(
                TT_F2_003,
                format!("Path not found: {}", root.display()),
                Some(root.display().to_string()),
            )),
        };
    }

    if let Err(e) = fs::metadata(root) {
        let code = if e.kind() == std::io::ErrorKind::PermissionDenied {
            permission_denied_code(host)
        } else {
            TT_F2_005
        };
        return GlobMatch {
            files: Vec::new(),
            file_count: 0,
            truncated: false,
            readable: false,
            status: SourceStatus::Error,
            error: Some(make_error(
                code,
                format!("Cannot read {}: {e}", root.display()),
                Some(root.display().to_string()),
            )),
        };
    }

    let mut matched: Vec<PathBuf> = Vec::new();
    let mut walk_error: Option<DiscoverError> = None;

    if !glob.contains('*') && !glob.contains('?') {
        let candidate = root.join(glob);
        if candidate.is_file() {
            matched.push(candidate);
        }
    } else {
        for entry in WalkDir::new(root).follow_links(false).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    let path = err
                        .path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| root.display().to_string());
                    let io_kind = err.io_error().map(|e| e.kind());
                    let code = if io_kind == Some(std::io::ErrorKind::PermissionDenied) {
                        permission_denied_code(host)
                    } else {
                        TT_F2_005
                    };
                    walk_error = Some(make_error(
                        code,
                        format!("Walk error under {}: {err}", root.display()),
                        Some(path),
                    ));
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = match entry.path().strip_prefix(root) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if glob_match(&rel_str, glob) {
                matched.push(entry.path().to_path_buf());
            }
        }
    }

    let file_count = matched.len() as u64;
    let truncated = cap > 0 && matched.len() > cap;
    let take_n = if cap == 0 {
        matched.len()
    } else {
        cap.min(matched.len())
    };
    let mut files: Vec<String> = matched
        .into_iter()
        .take(take_n)
        .map(|p| p.display().to_string())
        .collect();
    files.sort();

    if file_count == 0 {
        return GlobMatch {
            files,
            file_count: 0,
            truncated: false,
            readable: true,
            status: SourceStatus::Partial,
            error: walk_error,
        };
    }

    let status = if walk_error.is_some() {
        SourceStatus::Partial
    } else {
        SourceStatus::Ok
    };

    GlobMatch {
        files,
        file_count,
        truncated,
        readable: true,
        status,
        error: walk_error,
    }
}

/// Match relative paths against a restricted glob language (`**`, `*`, literals).
pub fn glob_match(path: &str, pattern: &str) -> bool {
    let path = path.replace('\\', "/");
    let pattern = pattern.replace('\\', "/");
    match_components(
        &path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
        &pattern.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
    )
}

fn match_components(path: &[&str], pat: &[&str]) -> bool {
    let mut pi = 0usize;
    let mut gi = 0usize;
    while gi < pat.len() {
        if pat[gi] == "**" {
            if gi + 1 == pat.len() {
                return true;
            }
            for start in pi..=path.len() {
                if match_components(&path[start..], &pat[gi + 1..]) {
                    return true;
                }
            }
            return false;
        }
        if pi >= path.len() {
            return false;
        }
        if !match_segment(path[pi], pat[gi]) {
            return false;
        }
        pi += 1;
        gi += 1;
    }
    pi == path.len()
}

fn match_segment(seg: &str, pat: &str) -> bool {
    if !pat.contains('*') && !pat.contains('?') {
        return seg == pat;
    }
    let mut si = 0usize;
    let pb: Vec<char> = pat.chars().collect();
    let sb: Vec<char> = seg.chars().collect();
    let mut pi = 0usize;
    while pi < pb.len() {
        match pb[pi] {
            '*' => {
                if pi + 1 == pb.len() {
                    return true;
                }
                for start in si..=sb.len() {
                    if match_segment(&sb[start..].iter().collect::<String>(), &pat[pi + 1..]) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if si >= sb.len() {
                    return false;
                }
                si += 1;
                pi += 1;
            }
            c => {
                if si >= sb.len() || sb[si] != c {
                    return false;
                }
                si += 1;
                pi += 1;
            }
        }
    }
    si == sb.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn glob_jsonl_under_projects() {
        assert!(glob_match("encoded-cwd/session-win.jsonl", "**/*.jsonl"));
        assert!(glob_match(
            "2026/09/11/rollout-2026-09-11T00-10-00-uuid.jsonl",
            "**/rollout-*.jsonl"
        ));
        assert!(!glob_match("notes.txt", "**/*.jsonl"));
        assert!(glob_match("state.vscdb", "state.vscdb"));
        assert!(glob_match("hash1/state.vscdb", "**/state.vscdb"));
        assert!(!glob_match("other.db", "**/rollout-*.jsonl"));
    }

    #[test]
    fn probe_cap_zero_is_unlimited_and_limit_truncates() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tt-bridge-glob-{nanos}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("projects/a")).unwrap();
        fs::write(root.join("projects/a/one.jsonl"), b"1").unwrap();
        fs::write(root.join("projects/a/two.jsonl"), b"2").unwrap();
        fs::write(root.join("projects/a/three.jsonl"), b"3").unwrap();

        let all = probe_glob(&root, "projects/**/*.jsonl", 0);
        assert_eq!(all.file_count, 3);
        assert_eq!(all.files.len(), 3);
        assert!(!all.truncated);

        let capped = probe_glob(&root, "projects/**/*.jsonl", 1);
        assert_eq!(capped.file_count, 3);
        assert_eq!(capped.files.len(), 1);
        assert!(capped.truncated);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_path_is_error_not_panic() {
        let root = std::env::temp_dir().join("tt-bridge-missing-path-nope");
        let _ = fs::remove_dir_all(&root);
        let m = probe_glob_for_host(&root, "**/*.jsonl", 0, SourceHost::Macos);
        assert_eq!(m.status, SourceStatus::Error);
        assert!(!m.readable);
        assert_eq!(m.file_count, 0);
        let err = m.error.expect("diagnostic");
        assert_eq!(err.code, "TT-F2-003");
        assert!(!err.next_step.is_empty());
    }

    #[test]
    fn permission_denied_on_macos_maps_to_tt_f10_fda() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tt-bridge-fda-{nanos}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&root).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&root, perms).unwrap();
        }
        let mac = probe_glob_for_host(&root, "**/*.jsonl", 0, SourceHost::Macos);
        let win = probe_glob_for_host(&root, "**/*.jsonl", 0, SourceHost::Windows);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&root).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&root, perms).unwrap();
        }
        let mac_err = mac.error.expect("macos diagnostic");
        assert_eq!(mac_err.code, "TT-F10-FDA", "{mac_err:?}");
        assert!(mac_err.next_step.contains("Full Disk Access"));
        assert!(mac_err.next_step.to_ascii_lowercase().contains("not required"));
        let win_err = win.error.expect("win diagnostic");
        assert_eq!(win_err.code, "TT-F2-004", "{win_err:?}");
        let _ = fs::remove_dir_all(&root);
    }
}
