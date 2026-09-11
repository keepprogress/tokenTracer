# OPEN-UI-4 回填 — by_model + usage_pool（帳本 → 儀表）

| 版本 | OPEN-UI-4 / LEDGER |
| 日期 | 2026-09-11 |
| 對照 | AC v1.3∪v1.3a；`design/ledger/by-model-v0.md` |

## 儀表可消費

| UI 需求 | 帳本來源 |
|---------|----------|
| 按 model 列 | `SpendSummary.by_model` 或 `spend by-model --currency USD` |
| Cursor 雙池 | `SpendSummary.by_usage_pool` 或 `spend by-pool` |
| Other Models 標籤 | `pool=other_models`（**非** model 字串） |
| notional 免責 | `pricing_mode` + `disclaimer` |

綁定契約可升 UI-BIND：Expanded 增加 by_model 列表＋可選按 pool 分組。數字仍禁止 UI 自算。

**狀態：帳本已回填；CLI／欄位隨 crate 落地後可 mock→真接。**
