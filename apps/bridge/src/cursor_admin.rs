//! Cursor official Admin API credential + source-mode stub (AC v1.4-cursor-official).
//!
//! Local-first: resolves Admin key from env/config, emits diagnostic errors, and
//! builds PARTIAL payloads for personal / no-key accounts. **No live HTTP** in this
//! stub. Contract: `docs/cursor-official-admin-v0.md`.
//!
//! Non-goals: L1 bubble/state.vscdb → USD/Spending; undocumented default on.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Exact contract message for personal / no-key (AC-F15 / F17).
pub const PERSONAL_NO_PUBLIC_USAGE_API_MSG: &str = "無公開個人 usage API";

/// Env var for Team / Enterprise Admin API key (preferred).
pub const ENV_ADMIN_API_KEY: &str = "TOKENTRACER_CURSOR_ADMIN_API_KEY";

/// Optional env pointing at a JSON config file with `cursor.admin_api_key`.
pub const ENV_ADMIN_CONFIG_PATH: &str = "TOKENTRACER_CURSOR_ADMIN_CONFIG";

/// Wire / product source modes (AC §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    OfficialAdmin,
    LocalEnrichment,
    UndocumentedDashboard,
}

impl SourceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceMode::OfficialAdmin => "official_admin",
            SourceMode::LocalEnrichment => "local_enrichment",
            SourceMode::UndocumentedDashboard => "undocumented_dashboard",
        }
    }
}

/// Opt-in flag for undocumented dashboard RPCs (AC-F16). **Default: Off.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UndocumentedDashboardFlag {
    #[default]
    Off,
    On,
}

impl UndocumentedDashboardFlag {
    pub fn is_on(self) -> bool {
        matches!(self, UndocumentedDashboardFlag::On)
    }

    pub fn is_off(self) -> bool {
        !self.is_on()
    }
}

/// Injectable credential resolution inputs (unit tests avoid real network / host env).
#[derive(Debug, Clone, Default)]
pub struct AdminCredentialConfig {
    /// When set, used instead of reading process env (tests inject here).
    pub env_api_key: Option<String>,
    /// Explicit config file path; if None, may fall back to `env_config_path`.
    pub config_path: Option<PathBuf>,
    /// Injected value of `TOKENTRACER_CURSOR_ADMIN_CONFIG` (tests).
    pub env_config_path: Option<String>,
    /// When true, read real process env for key / config path (production).
    pub use_process_env: bool,
    /// Undocumented dashboard opt-in; default Off.
    pub undocumented_dashboard: UndocumentedDashboardFlag,
}

/// Successfully resolved Admin key (never Display/Debug the raw secret in logs).
#[derive(Clone)]
pub struct ResolvedAdminKey {
    key: String,
    pub source: CredentialSource,
}

impl ResolvedAdminKey {
    pub fn from_parts(key: impl Into<String>, source: CredentialSource) -> Self {
        Self {
            key: key.into(),
            source,
        }
    }

    /// Borrow the secret for HTTP Basic auth (callers must not log it).
    pub fn secret(&self) -> &str {
        &self.key
    }

    pub fn redacted(&self) -> String {
        redact_secret(&self.key)
    }
}

impl std::fmt::Debug for ResolvedAdminKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedAdminKey")
            .field("key", &self.redacted())
            .field("source", &self.source)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    Env,
    ConfigFile,
}

/// Structured diagnostic (aligned with DiscoverError; AC-F14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminDiagnostic {
    pub code: String,
    pub message: String,
    pub next_step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redacted_hint: Option<String>,
}

impl AdminDiagnostic {
    pub fn missing_key() -> Self {
        Self {
            code: "TT-C14-MISSING-KEY".into(),
            message: "No Cursor Team Admin API key resolved (env or config)".into(),
            next_step: "Set TOKENTRACER_CURSOR_ADMIN_API_KEY, or config cursor.admin_api_key; personal accounts use PARTIAL (無公開個人 usage API).".into(),
            http_status: None,
            redacted_hint: None,
        }
    }

    pub fn unauthorized_401(redacted_hint: Option<String>) -> Self {
        Self {
            code: "TT-C14-401".into(),
            message: "Cursor Admin API returned 401 Unauthorized".into(),
            next_step: "Rotate or re-issue the Team Admin API key; confirm Basic auth uses KEY: with empty password.".into(),
            http_status: Some(401),
            redacted_hint,
        }
    }

    pub fn enterprise_required_403(redacted_hint: Option<String>) -> Self {
        Self {
            code: "TT-C14-403-ENTERPRISE".into(),
            message: "Cursor Admin API returned 403 Enterprise required".into(),
            next_step: "Use a Team/Enterprise Admin key with filtered-usage-events + /teams/spend access; personal Pro/Ultra has no public usage API (PARTIAL).".into(),
            http_status: Some(403),
            redacted_hint,
        }
    }

    pub fn config_unreadable(path: &Path, detail: impl Into<String>) -> Self {
        Self {
            code: "TT-C14-CONFIG-UNREADABLE".into(),
            message: format!(
                "Admin config unreadable at {}: {}",
                path.display(),
                detail.into()
            ),
            next_step: "Fix JSON shape {\"cursor\":{\"admin_api_key\":\"…\"}} and file permissions; or unset TOKENTRACER_CURSOR_ADMIN_CONFIG.".into(),
            http_status: None,
            redacted_hint: None,
        }
    }

    pub fn unsupported_undocumented() -> Self {
        Self {
            code: "TT-C14-UNSUPPORTED-UNDOCUMENTED".into(),
            message: "undocumented_dashboard is opt-in UNSUPPORTED / fragile".into(),
            next_step: "Keep flag off for official paths; if opt-in, label every emit UNSUPPORTED and never claim official_admin.".into(),
            http_status: None,
            redacted_hint: None,
        }
    }
}

/// Map an HTTP status from a future Admin client into a diagnostic (stub helper).
pub fn diagnostic_from_http_status(status: u16, redacted_hint: Option<String>) -> AdminDiagnostic {
    match status {
        401 => AdminDiagnostic::unauthorized_401(redacted_hint),
        403 => AdminDiagnostic::enterprise_required_403(redacted_hint),
        other => AdminDiagnostic {
            code: format!("TT-C14-HTTP-{other}"),
            message: format!("Cursor Admin API returned HTTP {other}"),
            next_step: "Inspect Admin API response body; confirm Team Admin key and endpoint (filtered-usage-events / teams/spend).".into(),
            http_status: Some(other),
            redacted_hint,
        },
    }
}

/// PARTIAL payload for 儀表／帳本 when personal / no Admin key (AC-F15／F17).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalPartialPayload {
    /// Always `"partial"` for this helper.
    pub status: String,
    pub source_mode: SourceMode,
    /// Exact contract string: `無公開個人 usage API`.
    pub message: String,
    pub message_code: String,
    pub align_targets: Vec<String>,
    pub billing_authoritative: bool,
    pub undocumented_dashboard: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<AdminDiagnostic>,
}

/// Result of choosing which source mode is active for this run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceModeSelection {
    pub mode: SourceMode,
    /// True when emit must be labeled UNSUPPORTED (undocumented opt-in).
    pub unsupported: bool,
    /// True only for OfficialAdmin with a resolved key.
    pub billing_authoritative: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial: Option<PersonalPartialPayload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<AdminDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct AdminConfigFile {
    cursor: Option<AdminConfigCursor>,
}

#[derive(Debug, Deserialize)]
struct AdminConfigCursor {
    admin_api_key: Option<String>,
}

/// Redact a secret for diagnostics (never full key).
pub fn redact_secret(secret: &str) -> String {
    let len = secret.len();
    if len == 0 {
        return "(empty)".into();
    }
    let prefix: String = secret.chars().take(4).collect();
    format!("{prefix}…(len={len})")
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

fn read_key_from_config_file(path: &Path) -> Result<String, AdminDiagnostic> {
    let raw = fs::read_to_string(path)
        .map_err(|e| AdminDiagnostic::config_unreadable(path, e.to_string()))?;
    let parsed: AdminConfigFile = serde_json::from_str(&raw)
        .map_err(|e| AdminDiagnostic::config_unreadable(path, e.to_string()))?;
    match parsed
        .cursor
        .and_then(|c| c.admin_api_key)
        .and_then(|k| non_empty(Some(k)))
    {
        Some(k) => Ok(k),
        None => Err(AdminDiagnostic::config_unreadable(
            path,
            "missing cursor.admin_api_key",
        )),
    }
}

/// Resolve Admin API key: env → optional config file → missing.
///
/// Does **not** contact the network. On missing key returns `Err(TT-C14-MISSING-KEY)`;
/// callers that want the personal PARTIAL path should use [`resolve_or_personal_partial`].
pub fn resolve_admin_credential(
    cfg: &AdminCredentialConfig,
) -> Result<ResolvedAdminKey, AdminDiagnostic> {
    // 1) Env (injected or process)
    let env_key = non_empty(cfg.env_api_key.clone()).or_else(|| {
        if cfg.use_process_env {
            non_empty(std::env::var(ENV_ADMIN_API_KEY).ok())
        } else {
            None
        }
    });
    if let Some(key) = env_key {
        return Ok(ResolvedAdminKey::from_parts(key, CredentialSource::Env));
    }

    // 2) Config file path (explicit → injected env path → process env path)
    let path = cfg.config_path.clone().or_else(|| {
        non_empty(cfg.env_config_path.clone())
            .or_else(|| {
                if cfg.use_process_env {
                    non_empty(std::env::var(ENV_ADMIN_CONFIG_PATH).ok())
                } else {
                    None
                }
            })
            .map(PathBuf::from)
    });

    if let Some(path) = path {
        let key = read_key_from_config_file(&path)?;
        return Ok(ResolvedAdminKey::from_parts(
            key,
            CredentialSource::ConfigFile,
        ));
    }

    Err(AdminDiagnostic::missing_key())
}

/// Build the PARTIAL personal payload (AC-F15／F17).
pub fn personal_partial_payload() -> PersonalPartialPayload {
    PersonalPartialPayload {
        status: "partial".into(),
        source_mode: SourceMode::LocalEnrichment,
        message: PERSONAL_NO_PUBLIC_USAGE_API_MSG.into(),
        message_code: "TT-C14-NO-PERSONAL-USAGE-API".into(),
        align_targets: vec![
            "cursor_models_percent".into(),
            "other_models_percent".into(),
            "reset".into(),
            "on_demand".into(),
        ],
        billing_authoritative: false,
        undocumented_dashboard: false,
        diagnostics: vec![AdminDiagnostic::missing_key()],
    }
}

/// Select active source mode given credential presence + undocumented flag.
///
/// **Invariant:** local enrichment / L1 never yields `OfficialAdmin`.
/// Undocumented opt-in yields `UndocumentedDashboard` with `unsupported: true`.
pub fn select_source_mode(
    cfg: &AdminCredentialConfig,
    key_present: bool,
) -> SourceModeSelection {
    if key_present {
        return SourceModeSelection {
            mode: SourceMode::OfficialAdmin,
            unsupported: false,
            billing_authoritative: true,
            partial: None,
            diagnostics: vec![],
        };
    }

    if cfg.undocumented_dashboard.is_on() {
        return SourceModeSelection {
            mode: SourceMode::UndocumentedDashboard,
            unsupported: true,
            billing_authoritative: false,
            partial: None,
            diagnostics: vec![AdminDiagnostic::unsupported_undocumented()],
        };
    }

    let partial = personal_partial_payload();
    SourceModeSelection {
        mode: SourceMode::LocalEnrichment,
        unsupported: false,
        billing_authoritative: false,
        partial: Some(partial),
        diagnostics: vec![AdminDiagnostic::missing_key()],
    }
}

/// Resolve key or fall back to personal PARTIAL selection (no network).
pub fn resolve_or_personal_partial(cfg: &AdminCredentialConfig) -> SourceModeSelection {
    match resolve_admin_credential(cfg) {
        Ok(_key) => select_source_mode(cfg, true),
        Err(_) => select_source_mode(cfg, false),
    }
}

/// Explicit guard: L1 local enrichment must never be classified as official_admin.
pub fn mode_for_local_enrichment_only() -> SourceMode {
    SourceMode::LocalEnrichment
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn env_key_present_resolves_official_admin() {
        let cfg = AdminCredentialConfig {
            env_api_key: Some("test-admin-key-abc".into()),
            use_process_env: false,
            undocumented_dashboard: UndocumentedDashboardFlag::Off,
            ..Default::default()
        };
        let key = resolve_admin_credential(&cfg).expect("key");
        assert_eq!(key.source, CredentialSource::Env);
        assert!(key.redacted().contains("test"), "{}", key.redacted());
        assert!(!key.redacted().contains("test-admin-key-abc"));
        let sel = resolve_or_personal_partial(&cfg);
        assert_eq!(sel.mode, SourceMode::OfficialAdmin);
        assert!(sel.billing_authoritative);
        assert!(sel.partial.is_none());
    }

    #[test]
    fn missing_key_yields_partial_personal_path() {
        let cfg = AdminCredentialConfig {
            use_process_env: false,
            undocumented_dashboard: UndocumentedDashboardFlag::Off,
            ..Default::default()
        };
        let err = resolve_admin_credential(&cfg).expect_err("missing");
        assert_eq!(err.code, "TT-C14-MISSING-KEY");

        let sel = resolve_or_personal_partial(&cfg);
        assert_eq!(sel.mode, SourceMode::LocalEnrichment);
        assert!(!sel.billing_authoritative);
        let p = sel.partial.expect("partial");
        assert_eq!(p.status, "partial");
        assert_eq!(p.message, PERSONAL_NO_PUBLIC_USAGE_API_MSG);
        assert_eq!(p.message, "無公開個人 usage API");
        assert!(!p.billing_authoritative);
        assert!(!p.undocumented_dashboard);
        assert!(p.align_targets.contains(&"cursor_models_percent".into()));
    }

    #[test]
    fn undocumented_dashboard_defaults_off() {
        let flag = UndocumentedDashboardFlag::default();
        assert!(flag.is_off());
        let cfg = AdminCredentialConfig::default();
        assert!(cfg.undocumented_dashboard.is_off());

        let sel = select_source_mode(&cfg, false);
        assert_ne!(sel.mode, SourceMode::UndocumentedDashboard);
        assert!(!sel.unsupported);
    }

    #[test]
    fn undocumented_opt_in_is_unsupported_not_official() {
        let cfg = AdminCredentialConfig {
            use_process_env: false,
            undocumented_dashboard: UndocumentedDashboardFlag::On,
            ..Default::default()
        };
        let sel = select_source_mode(&cfg, false);
        assert_eq!(sel.mode, SourceMode::UndocumentedDashboard);
        assert!(sel.unsupported);
        assert!(!sel.billing_authoritative);
        assert_ne!(sel.mode, SourceMode::OfficialAdmin);
        assert_eq!(
            sel.diagnostics[0].code,
            "TT-C14-UNSUPPORTED-UNDOCUMENTED"
        );
    }

    #[test]
    fn never_treat_l1_as_official_admin() {
        // L1-only path helper must stay LocalEnrichment (OPEN-C2 / AC-F15 BLOCK).
        assert_eq!(
            mode_for_local_enrichment_only(),
            SourceMode::LocalEnrichment
        );
        let cfg = AdminCredentialConfig {
            use_process_env: false,
            undocumented_dashboard: UndocumentedDashboardFlag::Off,
            ..Default::default()
        };
        let sel = select_source_mode(&cfg, false);
        assert_eq!(sel.mode, SourceMode::LocalEnrichment);
        assert_ne!(sel.mode, SourceMode::OfficialAdmin);
        assert!(!sel.billing_authoritative);
        // Selecting with key_present=false must not promote L1 → official_admin
        // even if a caller mistakenly conflates local DB with Admin.
        assert!(sel.partial.is_some());
    }

    #[test]
    fn config_file_key_second_in_resolution_order() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"{{"cursor":{{"admin_api_key":"cfg-key-xyz"}}}}"#
        )
        .unwrap();
        let cfg = AdminCredentialConfig {
            env_api_key: None,
            config_path: Some(file.path().to_path_buf()),
            use_process_env: false,
            ..Default::default()
        };
        let key = resolve_admin_credential(&cfg).expect("cfg key");
        assert_eq!(key.source, CredentialSource::ConfigFile);
        assert_eq!(key.secret(), "cfg-key-xyz");
    }

    #[test]
    fn env_wins_over_config_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"{{"cursor":{{"admin_api_key":"cfg-key"}}}}"#
        )
        .unwrap();
        let cfg = AdminCredentialConfig {
            env_api_key: Some("env-key".into()),
            config_path: Some(file.path().to_path_buf()),
            use_process_env: false,
            ..Default::default()
        };
        let key = resolve_admin_credential(&cfg).unwrap();
        assert_eq!(key.source, CredentialSource::Env);
        assert_eq!(key.secret(), "env-key");
    }

    #[test]
    fn http_401_and_403_diagnostics() {
        let d401 = diagnostic_from_http_status(401, Some("ab12…(len=16)".into()));
        assert_eq!(d401.code, "TT-C14-401");
        assert_eq!(d401.http_status, Some(401));
        let d403 = diagnostic_from_http_status(403, None);
        assert_eq!(d403.code, "TT-C14-403-ENTERPRISE");
        assert_eq!(d403.http_status, Some(403));
        assert!(d403.message.contains("Enterprise"));
    }

    #[test]
    fn redact_never_emits_full_key() {
        let full = "super-secret-admin-key-value";
        let r = redact_secret(full);
        assert!(!r.contains(full));
        assert!(r.contains("len="));
    }
}
