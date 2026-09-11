# Cursor official Admin bridge contract v0

Canonical design lives in the bridge docs (same revision):

**[`apps/bridge/docs/cursor-official-admin-v0.md`](../../apps/bridge/docs/cursor-official-admin-v0.md)**

Stub module: `tokentracer_bridge::cursor_admin`  
Spec: AC v1.4-cursor-official (F14–F17)

Quick reference:

| Item | Value |
|------|-------|
| Credential order | `TOKENTRACER_CURSOR_ADMIN_API_KEY` → config `cursor.admin_api_key` → none |
| Missing / personal | PARTIAL + message **`無公開個人 usage API`** |
| Error codes | `TT-C14-MISSING-KEY`, `TT-C14-401`, `TT-C14-403-ENTERPRISE`, `TT-C14-CONFIG-UNREADABLE`, `TT-C14-UNSUPPORTED-UNDOCUMENTED` |
| `undocumented_dashboard` | **default off**; opt-in → UNSUPPORTED |
| Non-goals | L1-as-billing; undocumented default on |
