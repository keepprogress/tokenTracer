//! Local filesystem expand for DiscoverSource when `files[]` is absent/incomplete.

use super::discover::{DiscoverError, DiscoverSource};
use globset::{Glob, GlobSetBuilder};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ExpandOutcome {
    pub files: Vec<String>,
    pub file_count: u64,
    pub truncated: bool,
    pub errors: Vec<DiscoverError>,
    pub import_root: Option<PathBuf>,
}

/// Resolve the directory to walk for local expand (fixture / Linux hosts).
pub fn resolve_import_root(src: &DiscoverSource) -> Option<PathBuf> {
    if let Some(meta) = &src.meta {
        for key in ["import_path", "posix_path"] {
            if let Some(p) = meta.get(key) {
                let path = PathBuf::from(p);
                if path.is_dir() {
                    return Some(path);
                }
            }
        }
    }

    let root = PathBuf::from(&src.root_path);
    if root.is_dir() {
        return Some(root);
    }

    // wsl:<Distro>:<posix-abs> → try posix abs on Linux fixture hosts
    if let Some(posix) = strip_wsl_root(&src.root_path) {
        let path = PathBuf::from(posix);
        if path.is_dir() {
            return Some(path);
        }
    }

    None
}

/// Strip `wsl:<Distro>:` prefix, leaving the posix absolute path.
pub fn strip_wsl_root(root_path: &str) -> Option<&str> {
    let rest = root_path.strip_prefix("wsl:")?;
    // rest = Distro:/posix...  (distro may contain hyphens, not colons typically)
    let (_distro, posix) = rest.split_once(':')?;
    if posix.starts_with('/') {
        Some(posix)
    } else {
        None
    }
}

/// Expand `glob` under import root. `limit == 0` means unlimited.
pub fn expand_files_locally(src: &DiscoverSource, limit: u64) -> ExpandOutcome {
    let Some(root) = resolve_import_root(src) else {
        return ExpandOutcome {
            files: Vec::new(),
            file_count: 0,
            truncated: false,
            errors: vec![DiscoverError {
                code: "TT-F2-003".into(),
                message: format!(
                    "Import root not found for source {} (root_path={})",
                    src.id, src.root_path
                ),
                next_step: "Verify agent install / fixture paths; set meta.import_path".into(),
                path: Some(src.root_path.clone()),
            }],
            import_root: None,
        };
    };

    let matcher = match build_glob_matcher(&src.glob) {
        Ok(m) => m,
        Err(e) => {
            return ExpandOutcome {
                files: Vec::new(),
                file_count: 0,
                truncated: false,
                errors: vec![DiscoverError {
                    code: "TT-F2-005".into(),
                    message: format!("Invalid glob '{}': {e}", src.glob),
                    next_step: "Fix DiscoverSource.glob".into(),
                    path: Some(src.id.clone()),
                }],
                import_root: Some(root),
            };
        }
    };

    let mut matched: Vec<String> = Vec::new();
    for entry in WalkDir::new(&root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let full = entry.path();
        let rel = match full.strip_prefix(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // Normalize to forward slashes for glob match
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if matcher.is_match(&rel_str) {
            matched.push(full.to_string_lossy().replace('\\', "/"));
        }
    }
    matched.sort();

    let total = matched.len() as u64;
    let mut truncated = false;
    let mut errors = Vec::new();
    let files = if limit > 0 && total > limit {
        truncated = true;
        errors.push(DiscoverError::truncated_list(Some(src.id.clone())));
        matched.into_iter().take(limit as usize).collect()
    } else {
        matched
    };

    ExpandOutcome {
        files,
        file_count: total,
        truncated,
        errors,
        import_root: Some(root),
    }
}

fn build_glob_matcher(pattern: &str) -> Result<globset::GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    // globset treats `**` correctly; also accept patterns without leading **/
    let glob = Glob::new(pattern).map_err(|e| e.to_string())?;
    builder.add(glob);
    // Also try matching basename-only patterns against full relative path (already covered)
    builder.build().map_err(|e| e.to_string())
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::discover::DiscoverSource;
    use std::collections::HashMap;

    #[test]
    fn strip_wsl_root_ok() {
        assert_eq!(
            strip_wsl_root("wsl:Ubuntu-Work:/home/t3261/.claude"),
            Some("/home/t3261/.claude")
        );
        assert_eq!(strip_wsl_root("/home/x"), None);
    }

    #[test]
    fn resolve_prefers_import_path() {
        let fixtures = "/workspace/tokenTracer-bridge/tests/fixtures/wsl_ubuntu/home/t3261/.claude";
        if !std::path::Path::new(fixtures).is_dir() {
            return;
        }
        let mut meta = HashMap::new();
        meta.insert("import_path".into(), fixtures.into());
        meta.insert("posix_path".into(), "/home/t3261/.claude".into());
        let src = DiscoverSource {
            id: "x".into(),
            agent: "claude_code".into(),
            evidence_card: None,
            host: "wsl2".into(),
            root_path: "wsl:Ubuntu-Work:/home/t3261/.claude".into(),
            glob: "projects/**/*.jsonl".into(),
            files: None,
            file_count: 1,
            truncated: None,
            readable: true,
            status: "ok".into(),
            meta: Some(meta),
        };
        let root = resolve_import_root(&src).unwrap();
        assert_eq!(root, PathBuf::from(fixtures));
    }

    #[test]
    fn expand_claude_fixture_glob() {
        let root = "/workspace/tokenTracer-bridge/tests/fixtures/win_home/.claude";
        if !std::path::Path::new(root).is_dir() {
            return;
        }
        let src = DiscoverSource {
            id: "x".into(),
            agent: "claude_code".into(),
            evidence_card: None,
            host: "windows".into(),
            root_path: root.into(),
            glob: "projects/**/*.jsonl".into(),
            files: None,
            file_count: 2,
            truncated: None,
            readable: true,
            status: "ok".into(),
            meta: None,
        };
        let out = expand_files_locally(&src, 0);
        assert!(!out.truncated);
        assert_eq!(out.files.len(), 2);
        assert_eq!(out.file_count, 2);
    }
}
