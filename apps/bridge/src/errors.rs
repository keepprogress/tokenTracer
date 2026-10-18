//! Diagnostic error codes for AC-F1 / AC-F2 / AC-F10 (bridge).

use crate::types::DiscoverError;

/// Catalog entry: code + default next_step template.
#[derive(Debug, Clone, Copy)]
pub struct ErrorCode {
    pub code: &'static str,
    pub default_message: &'static str,
    pub next_step: &'static str,
}

pub const TT_F1_001: ErrorCode = ErrorCode {
    code: "TT-F1-001",
    default_message: "Install failed",
    next_step: "Re-run the platform installer from README; check write permission to the install directory; capture the installer log and retry.",
};

pub const TT_F1_002: ErrorCode = ErrorCode {
    code: "TT-F1-002",
    default_message: "Start failed",
    next_step: "Confirm the binary exists on PATH; check tray/host process logs; try `tokentracer-bridge --help` from an elevated/normal user shell.",
};

pub const TT_F1_003: ErrorCode = ErrorCode {
    code: "TT-F1-003",
    default_message: "Missing dependency",
    next_step: "Install the missing runtime dependency listed in the message (e.g. WebView2 on Windows, WSL optional feature), then restart tokenTracer.",
};

pub const TT_F2_001: ErrorCode = ErrorCode {
    code: "TT-F2-001",
    default_message: "WSL not installed",
    next_step: "Install WSL2 (`wsl --install` or enable Virtual Machine Platform + WSL), reboot if prompted, then re-run discovery.",
};

pub const TT_F2_002: ErrorCode = ErrorCode {
    code: "TT-F2-002",
    default_message: "WSL distro list failed",
    next_step: "Run `wsl.exe -l -v` manually; repair WSL if the command errors; ensure the bridge can invoke `wsl.exe` on PATH.",
};

pub const TT_F2_003: ErrorCode = ErrorCode {
    code: "TT-F2-003",
    default_message: "Path not found",
    next_step: "Verify the agent is installed on that host (Win vs WSL trees are separate); confirm the expected root exists, then re-run discovery.",
};

pub const TT_F2_004: ErrorCode = ErrorCode {
    code: "TT-F2-004",
    default_message: "Permission denied",
    next_step: "Grant read access to the path for the current user; on WSL ensure the Windows user can read `\\\\wsl$\\<Distro>\\home\\...` or use `wsl -d <Distro> --` probes.",
};

pub const TT_F2_005: ErrorCode = ErrorCode {
    code: "TT-F2-005",
    default_message: "Path unreadable",
    next_step: "Check the path is not locked/corrupt; for SQLite open read-only; close exclusive locks if safe, then retry discovery.",
};

pub const TT_F2_006: ErrorCode = ErrorCode {
    code: "TT-F2-006",
    default_message: "File list truncated",
    next_step: "Re-run with a higher `--limit N`, or `--limit 0` for unlimited expand (import full list). Do not treat a truncated `files[]` as complete.",
};

/// macOS Full Disk Access / privacy permission denied (AC-F10).
pub const TT_F10_FDA: ErrorCode = ErrorCode {
    code: "TT-F10-FDA",
    default_message: "macOS permission denied (Full Disk Access troubleshooting)",
    next_step: "System Settings → Privacy & Security → Full Disk Access → enable tokenTracer; FDA is troubleshooting only, not required for a normal install. Re-run discover.",
};

/// Stable numeric twin of [`TT_F10_FDA`] (same next_step contract).
pub const TT_F10_001: ErrorCode = ErrorCode {
    code: "TT-F10-001",
    default_message: "macOS permission denied reading agent path",
    next_step: "System Settings → Privacy & Security → Full Disk Access → enable tokenTracer; FDA is troubleshooting only, not required for a normal install. Re-run discover.",
};

pub fn make_error(code: ErrorCode, message: impl Into<String>, path: Option<String>) -> DiscoverError {
    DiscoverError {
        code: code.code.to_string(),
        message: message.into(),
        next_step: code.next_step.to_string(),
        path,
    }
}

/// Remap a generic permission-denied code to TT-F10 when the host is macOS.
pub fn permission_denied_code(host: crate::types::SourceHost) -> ErrorCode {
    match host {
        crate::types::SourceHost::Macos => TT_F10_FDA,
        _ => TT_F2_004,
    }
}

/// True when `next_step` documents FDA as troubleshooting-only (not install prerequisite).
pub fn fda_is_troubleshooting_only(code: &ErrorCode) -> bool {
    let s = code.next_step.to_ascii_lowercase();
    s.contains("troubleshooting only") && s.contains("not required")
}

/// All AC-F1/F2/F10 codes defined for this POC (for docs / tests).
pub fn catalog() -> &'static [ErrorCode] {
    &[
        TT_F1_001, TT_F1_002, TT_F1_003, TT_F2_001, TT_F2_002, TT_F2_003, TT_F2_004,
        TT_F2_005, TT_F2_006, TT_F10_FDA, TT_F10_001,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceHost;

    #[test]
    fn macos_permission_maps_to_tt_f10_fda() {
        assert_eq!(permission_denied_code(SourceHost::Macos).code, "TT-F10-FDA");
        assert_eq!(permission_denied_code(SourceHost::Windows).code, "TT-F2-004");
        assert_eq!(permission_denied_code(SourceHost::Wsl2).code, "TT-F2-004");
    }

    #[test]
    fn fda_next_step_is_troubleshooting_not_install_prereq() {
        assert!(fda_is_troubleshooting_only(&TT_F10_FDA));
        assert!(fda_is_troubleshooting_only(&TT_F10_001));
        assert!(TT_F10_FDA.next_step.contains("System Settings"));
        assert!(TT_F10_FDA.next_step.contains("Full Disk Access"));
    }
}
