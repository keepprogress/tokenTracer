# AC v1.3b — AC-F3 註記修正（訂正 `/mnt/c`／+347 誤讀）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.3b**（修訂稿） |
| 狀態 | **AC v1.3b — 待核准** |
| 類型 | 驗收語意修正 |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 修訂原因 | 橋樑抽查 Full AFTER `events.json`：`host_os=windows` **21886**＝Win-native；`wsl2` 12596；合計 34482。先前「+347＝Win 漏吞」**不成立**。 |

---

## 0. 廢止的錯誤依據

**廢止：** 以 Full AFTER 相對 Work-only 僅 **+347** 推斷「`/mnt/c` 未完整匯入 Win」。

**事實（NB-T3261 產物）：**
| 切片 | events |
|------|--------|
| Work-only（WSL importer） | 34135（皆 wsl2 樹） |
| Full AFTER（WSL importer＋`/mnt/c`） | 34482＝**windows 21886**＋**wsl2 12596** |
| Win-native（PE `spend.exe`） | **21886**（windows only） |

→ Full 內 **Win upsert 完整計數已與 Win-native 對齊（21886）**。  
→ +347 假象來自合併後 **wsl2 從 34135→12596 下降**（疑跨 host／穩定 id **去重**），不是 Win 沒進。

---

## 1. 鎖定文案（請代核）

**AC-F3 註記：**

1. **`/mnt/c`（WSL 讀 Win 樹）**  
   - **可**作為 Win 樹 **資料面 upsert** 證據，當且僅當報告可拆出 `host_os=windows`（或等價）計數，並與同機 Win-only／Win-native 切片對齊（容差／去重規則寫明）。  
   - 本機已證：Full AFTER windows＝**21886**＝Win-native。

2. **Win-native importer（真 Win 進程）**  
   - 仍為關閉「**Win-native host importer**」UNTESTED 的**進程／路徑契約**證據（PE `spend`、非 WSL importer）。  
   - **依據＝進程契約**，不是「+347 vs 21886 漏吞」。  
   - 帳本已對核 Win-NATIVE 21886 → 預驗可摘該 UNTESTED（非總 PASS）。

3. **禁止**再用「Full−Work＝+347」單獨斷言 Win 完整性或漏匯。

4. **OPEN-F3-DEDUP（新）**：Work-only 34135 → Full 內 wsl2 **12596**。帳本／橋樑根因（2026-09-11）：跨 host **共用 Claude／Codex vendor id** 去重（合併順序先 Win 後 WSL → WSL 被吃掉）。修正方向：namespaced id（例 `source.id::vendor_id`）。須可重現；不擋 Win-native 關閉；**禁止**用 Full−Work 總差分當 Win 完整性。

5. Stopped Ubuntu 跨 distro、`source_id`≠已讀、F7′／F10 — 本補丁不改。

---

## 2. 相對 v1.3b 初稿

| 初稿 | 修訂 |
|------|------|
| `/mnt/c` 不當完整 Win upsert | **改**：資料面完整可成立（本機 21886 對齊）；另查 wsl2 去重 |
| 以 +347≪21886 證漏吞 | **廢止**該因果 |
| Win-native 為完整 Win 唯一依據 | **改**：Win-native＝**進程契約**；資料面可以 Full 內 windows 計數對齊 |

---

## 3. 矩陣建議（預驗）

| 項 | 狀態 |
|----|------|
| F3 WSL-Work upsert | OK |
| F3 Win 資料面（Full 內 windows＝21886 或 Win-native） | OK（有證） |
| F3 `/mnt/c` 可見性 | OK（且本機資料面完整） |
| Win-native importer 進程 | **可關 UNTESTED**（帳本已對核；規格不自 PASS） |
| OPEN-F3-DEDUP | 開（wsl2 34135→12596） |
| Stopped Ubuntu | UNTESTED |
| F7′／F10 | 進行中／HOLD |

---

## 4. 交接

待指揮官代核本修訂稿；適配驗改鎖版。路徑：`specs/AC-v1.3b.md`。

---

**AC v1.3b — 待核准（修訂稿）**
