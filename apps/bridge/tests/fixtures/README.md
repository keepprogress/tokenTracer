# Discovery fixtures (synthetic)

Synthetic agent trees for Linux-box dual-scan tests. **No secrets**: no `.credentials.json`, `auth.json`, or Cursor auth tokens.

Layout:
- `win_home/` — simulates `%USERPROFILE%` (`.claude`, `.codex`, `.cursor`)
- `win_appdata/` — simulates `%APPDATA%` (`Cursor/User/...`)
- `wsl_ubuntu/home/t3261/` — simulates WSL Ubuntu Linux home
- `wsl_distros.json` — injectable distro list for AC-F2 simulation
