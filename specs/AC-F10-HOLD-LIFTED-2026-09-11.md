# AC-F10 — 解除優先 HOLD 註記（不改已核條款）

| 欄位 | 值 |
|------|-----|
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 觸發 | 人類核准 **F10 macOS**（結束優先 HOLD；經指揮官轉達） |
| 閘門基準 | **已核 [AC v1.2](./AC-v1.2.md)** ＋ **[AC v1.2b](./AC-v1.2b.md)** |
| 本文件性質 | **優先／排程註記**；**不修改**已核 AC 正文條款 |

---

## 1. 閘門基準（驗收官／實作）

| 項 | 基準 |
|----|------|
| **AC-F10** | `.dmg` 拖曳安裝（README）；權限失敗可診斷步驟；Apple Silicon **必過**；Intel **best-effort**（不擋 PASS） |
| **AC-F7′（macOS）** | 選單列圖示＋迷你面板；收合／展開資訊對齊 Win F7′（今日／歷史 USD、佔比、趨勢、區間、新鮮度、PARTIAL）；數字與 `spend` CLI 一致 |
| **AC-F1′** | macOS 13 Ventura+ 可裝可開 |
| **AC-F2′／矩陣** | macOS 欄依 v1.2b（文件級 CONFIRMED*）；無實機 dump 不得虛報；升格見 v1.2c |
| **繼承** | F12 notional 分欄、禁假 Other（v1.3a）；單機聚合、無 Win↔Mac 同步（OPEN-M3） |

**最終 PASS＝驗收官；規格不自 PASS。**

---

## 2. 驗收 Checklist（F10 開工用；對照已核 AC）

- [ ] `.dmg` 安裝路徑寫進 README；Apple Silicon 乾淨機可裝可開  
- [ ] 權限不足時 UI／文件給可照做步驟；不得靜默 0 資料  
- [ ] 選單列迷你面板：收合主數字、展開對齊 F7′ 資訊架構  
- [ ] 面板數字 vs `spend` CLI（同 F6 精神）  
- [ ] F12 notional disclaimer（若顯示花費估計）  
- [ ] 矩陣 macOS 欄無虛報（v1.2b／證據卡）  
- [ ] Intel = best-effort 註記（不擋 PASS）

---

## 3. 仍 HOLD（本註記不解除）

| ID | 狀態 |
|----|------|
| OPEN-C6／C8／C9 | PARTIAL／HOLD（見 AC v1.4 §6） |
| AC v1.3b | 待人類核 |
| Stopped Ubuntu 雙 distro | UNTESTED |
| v1.2c | macOS **實機** dump 升格 |

---

## 4. 交接

| 對象 | 動作 |
|------|------|
| 儀表 | macOS 選單列迷你面板（對齊 F7′） |
| 橋樑／打包 | `.dmg`、權限診斷、Apple Silicon 必過 |
| 探針 | 缺證再補 macOS 卡；不虛報 |
| 指揮官 | 輕 docs PR 合入本註記＋SPEC-INDEX；再派實作 |

**AC-F10 優先 HOLD — 已解除（2026-09-11）**
