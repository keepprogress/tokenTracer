/**
 * Dev/preview bridge: shell-host IPC names → real `spend` CLI JSON.
 * Prefer prebuilt target/{debug,release}/spend; else `cargo run -p pricing --bin spend`.
 * Falls back is handled by the UI when this middleware returns 503.
 *
 * Env:
 *   LEDGER_ROOT / TOKENTRACER_LEDGER_ROOT — cargo workspace root
 *   TOKENTRACER_SPEND_BIN — path to spend binary
 *   TOKENTRACER_EVENTS_RANGED — default ranged fixture
 *   TOKENTRACER_EVENTS_POOL — by-pool (non-ranged) fixture
 *   TOKENTRACER_EVENTS_MODEL — by-model fixture
 *   TOKENTRACER_IMPORT_STATE — import-meta state path
 *   TOKENTRACER_FORCE_FIXTURES=1 — middleware always 503 (UI uses static mocks)
 *   TOKENTRACER_SPENDING_ALIGN — OPEN-BIND SpendingAlign fixture (B fallback only)
 *   TOKENTRACER_SPENDING_ALIGN_STATE — `.token-tracer/spending-align.json` for CLI --state
 *   TOKENTRACER_ADMIN_EVENTS / TOKENTRACER_ADMIN_SPEND — optional F14 reconcile stub
 */
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
/** apps/ui → repo root */
const DEFAULT_ROOT = path.resolve(__dirname, "../../..");

const RANGES = new Set(["today", "all", "7d", "30d", "90d"]);
const PANEL_RANGES = new Set(["all", "7d", "30d", "90d"]);
const CURRENCIES = new Set(["USD", "TWD"]);

export function resolveLedgerRoot() {
  return (
    process.env.TOKENTRACER_LEDGER_ROOT ||
    process.env.LEDGER_ROOT ||
    DEFAULT_ROOT
  );
}

export function resolveSpendLauncher(root = resolveLedgerRoot()) {
  const envBin = process.env.TOKENTRACER_SPEND_BIN;
  if (envBin && fs.existsSync(envBin)) {
    return { cmd: envBin, argsPrefix: [], cwd: root, mode: "bin" };
  }
  for (const rel of ["target/release/spend", "target/debug/spend"]) {
    const p = path.join(root, rel);
    if (fs.existsSync(p)) {
      return { cmd: p, argsPrefix: [], cwd: root, mode: "bin" };
    }
  }
  return {
    cmd: "cargo",
    argsPrefix: ["run", "-q", "-p", "pricing", "--bin", "spend", "--"],
    cwd: root,
    mode: "cargo",
  };
}

function fixturePath(root, envKey, rel) {
  const override = process.env[envKey];
  if (override) return path.isAbsolute(override) ? override : path.join(root, override);
  return path.join(root, rel);
}

export function defaultPaths(root = resolveLedgerRoot()) {
  return {
    ranged: fixturePath(
      root,
      "TOKENTRACER_EVENTS_RANGED",
      "fixtures/ac-v1.3a/cursor-pools-ranged.json",
    ),
    pool: fixturePath(
      root,
      "TOKENTRACER_EVENTS_POOL",
      "fixtures/ac-v1.3a/cursor-pools.json",
    ),
    model: fixturePath(
      root,
      "TOKENTRACER_EVENTS_MODEL",
      "fixtures/ac-v1.3/by-model.json",
    ),
    importState: fixturePath(
      root,
      "TOKENTRACER_IMPORT_STATE",
      ".token-tracer/import-meta.json",
    ),
    spendingAlign: fixturePath(
      root,
      "TOKENTRACER_SPENDING_ALIGN",
      "apps/ui/src/mock/spending-align-manual-p1.json",
    ),
    spendingAlignState: fixturePath(
      root,
      "TOKENTRACER_SPENDING_ALIGN_STATE",
      ".token-tracer/spending-align.json",
    ),
  };
}

/** Extract first top-level JSON value from possibly mixed CLI stdout. */
export function extractJson(raw) {
  const startObj = raw.indexOf("{");
  const startArr = raw.indexOf("[");
  let start = -1;
  if (startObj < 0) start = startArr;
  else if (startArr < 0) start = startObj;
  else start = Math.min(startObj, startArr);
  if (start < 0) throw new Error("no JSON found in spend CLI stdout");

  const open = raw[start];
  const close = open === "{" ? "}" : "]";
  let depth = 0;
  let inStr = false;
  let esc = false;
  let end = -1;
  for (let i = start; i < raw.length; i++) {
    const ch = raw[i];
    if (inStr) {
      if (esc) esc = false;
      else if (ch === "\\") esc = true;
      else if (ch === '"') inStr = false;
      continue;
    }
    if (ch === '"') inStr = true;
    else if (ch === open) depth++;
    else if (ch === close) {
      depth--;
      if (depth === 0) {
        end = i + 1;
        break;
      }
    }
  }
  if (end < 0) throw new Error("unterminated JSON in spend CLI stdout");
  return JSON.parse(raw.slice(start, end));
}

function runSpend(cliArgs, { timeoutMs = 120_000 } = {}) {
  const root = resolveLedgerRoot();
  const launcher = resolveSpendLauncher(root);
  const args = [...launcher.argsPrefix, ...cliArgs];
  return new Promise((resolve, reject) => {
    const child = spawn(launcher.cmd, args, {
      cwd: launcher.cwd,
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      reject(new Error(`spend CLI timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (d) => {
      stdout += d;
    });
    child.stderr.on("data", (d) => {
      stderr += d;
    });
    child.on("error", (err) => {
      clearTimeout(timer);
      reject(err);
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      if (code !== 0) {
        reject(
          new Error(
            `spend exited ${code}: ${stderr.trim() || stdout.trim() || "(no output)"}`,
          ),
        );
        return;
      }
      try {
        resolve({ json: extractJson(stdout), launcher, stdout, stderr });
      } catch (e) {
        reject(
          new Error(
            `${e.message}; stdout head=${JSON.stringify(stdout.slice(0, 200))}`,
          ),
        );
      }
    });
  });
}

function parseQuery(url) {
  const u = new URL(url, "http://127.0.0.1");
  return u.searchParams;
}

function sendJson(res, status, body, extraHeaders = {}) {
  const payload = typeof body === "string" ? body : JSON.stringify(body);
  res.statusCode = status;
  res.setHeader("Content-Type", "application/json; charset=utf-8");
  res.setHeader("Cache-Control", "no-store");
  for (const [k, v] of Object.entries(extraHeaders)) res.setHeader(k, v);
  res.end(payload);
}

function sendError(res, status, message) {
  sendJson(res, status, { error: message, source: "cli" });
}

/**
 * Connect-style middleware for Vite configureServer / configurePreviewServer.
 * Mounts under /api/ipc/*
 */

/** Run spend and resolve on exit 0 without requiring JSON stdout (e.g. --help). */
function runSpendExitOk(cliArgs, { timeoutMs = 120_000 } = {}) {
  const root = resolveLedgerRoot();
  const launcher = resolveSpendLauncher(root);
  const args = [...launcher.argsPrefix, ...cliArgs];
  return new Promise((resolve, reject) => {
    const child = spawn(launcher.cmd, args, {
      cwd: launcher.cwd,
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      reject(new Error(`spend CLI timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (d) => {
      stderr += d;
    });
    child.on("error", (err) => {
      clearTimeout(timer);
      reject(err);
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      if (code !== 0) {
        reject(new Error(`spend exited ${code}: ${stderr.trim() || "(no output)"}`));
        return;
      }
      resolve(true);
    });
  });
}

/** True when `spend spending-align --help` succeeds (ledger OPEN-BIND CLI). */
let spendingAlignCliSupported = null;
async function probeSpendingAlignCli() {
  if (spendingAlignCliSupported != null) return spendingAlignCliSupported;
  try {
    await runSpendExitOk(["spending-align", "--help"], { timeoutMs: 60_000 });
    spendingAlignCliSupported = true;
  } catch {
    spendingAlignCliSupported = false;
  }
  return spendingAlignCliSupported;
}

function readJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

export function createSpendIpcMiddleware() {
  return async function spendIpcMiddleware(req, res, next) {
    const url = req.url || "";
    if (!url.startsWith("/api/ipc")) return next();

    if (process.env.TOKENTRACER_FORCE_FIXTURES === "1") {
      sendError(res, 503, "TOKENTRACER_FORCE_FIXTURES=1 — UI should use static mocks");
      return;
    }

    const root = resolveLedgerRoot();
    const paths = defaultPaths(root);
    const pathname = url.split("?")[0].replace(/\/+$/, "") || "/";
    const q = parseQuery(url);

    const currency = (q.get("currency") || "USD").toUpperCase();
    if (!CURRENCIES.has(currency)) {
      sendError(res, 400, `invalid currency: ${currency}`);
      return;
    }

    try {
      if (pathname === "/api/ipc/_meta" || pathname === "/api/ipc/meta") {
        const launcher = resolveSpendLauncher(root);
        sendJson(res, 200, {
          source: "cli",
          ledger_root: root,
          spend_mode: launcher.mode,
          spend_cmd: launcher.cmd,
          fixtures: paths,
        });
        return;
      }

      if (pathname === "/api/ipc/spend_total") {
        const range = q.get("range") || "all";
        if (!RANGES.has(range)) {
          sendError(res, 400, `invalid range: ${range}`);
          return;
        }
        const { json, launcher } = await runSpend([
          "by-pool",
          "--currency",
          currency,
          "--events",
          paths.ranged,
          "--range",
          range,
        ]);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "cli",
          "X-TokenTracer-Spend-Mode": launcher.mode,
        });
        return;
      }

      if (pathname === "/api/ipc/spend_series") {
        const grain = q.get("grain") || "day";
        const range = q.get("range") || "30d";
        if (grain !== "day") {
          sendError(res, 400, "v0 grain is day only");
          return;
        }
        if (!PANEL_RANGES.has(range)) {
          sendError(res, 400, `invalid series range: ${range}`);
          return;
        }
        const { json, launcher } = await runSpend([
          "series",
          "--grain",
          "day",
          "--currency",
          currency,
          "--events",
          paths.ranged,
          "--range",
          range,
        ]);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "cli",
          "X-TokenTracer-Spend-Mode": launcher.mode,
        });
        return;
      }

      if (pathname === "/api/ipc/spend_by_model") {
        const range = q.get("range") || "all";
        if (!RANGES.has(range)) {
          sendError(res, 400, `invalid range: ${range}`);
          return;
        }
        const { json, launcher } = await runSpend([
          "by-model",
          "--currency",
          currency,
          "--events",
          paths.model,
          "--range",
          range,
        ]);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "cli",
          "X-TokenTracer-Spend-Mode": launcher.mode,
        });
        return;
      }

      if (pathname === "/api/ipc/spend_by_pool") {
        const range = q.get("range") || "all";
        if (!RANGES.has(range)) {
          sendError(res, 400, `invalid range: ${range}`);
          return;
        }
        // Non-ranged dual-pool sample when range=all (matches refresh-from-ledger.sh);
        // otherwise use ranged fixture so chips stay consistent.
        const events = range === "all" ? paths.pool : paths.ranged;
        const { json, launcher } = await runSpend([
          "by-pool",
          "--currency",
          currency,
          "--events",
          events,
          "--range",
          range,
        ]);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "cli",
          "X-TokenTracer-Spend-Mode": launcher.mode,
        });
        return;
      }

      if (pathname === "/api/ipc/import_status") {
        const { json, launcher } = await runSpend([
          "import",
          "status",
          "--json",
          "--state",
          paths.importState,
        ]);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "cli",
          "X-TokenTracer-Spend-Mode": launcher.mode,
        });
        return;
      }

      if (pathname === "/api/ipc/spending_align") {
        // OPEN-BIND-S1 B surface: prefer live
        //   spend spending-align --json [--state .token-tracer/spending-align.json]
        // Fixture only on failure / missing bin / FORCE_FIXTURES. Never invent pct.
        const supported = await probeSpendingAlignCli();
        if (supported) {
          try {
            const statePath = paths.spendingAlignState;
            const args = ["spending-align", "--json", "--state", statePath];
            const { json, launcher } = await runSpend(args);
            sendJson(res, 200, json, {
              "X-TokenTracer-Source": "cli",
              "X-TokenTracer-Spend-Mode": launcher.mode,
              "X-TokenTracer-Spending-Align": "cli",
            });
            return;
          } catch (e) {
            // fall through to fixture
            const message = e instanceof Error ? e.message : String(e);
            console.warn("[spend-dev-bridge] spending-align CLI failed; fixture:", message);
          }
        }
        if (!fs.existsSync(paths.spendingAlign)) {
          sendError(res, 503, "spending-align fixture missing and CLI unsupported");
          return;
        }
        const json = readJsonFile(paths.spendingAlign);
        sendJson(res, 200, json, {
          "X-TokenTracer-Source": "fixture",
          "X-TokenTracer-Spending-Align": supported ? "cli-fallback-fixture" : "fixture-no-cli",
        });
        return;
      }

      // Optional F14 stub — only when env fixtures present; never blocks B-surface mock.
      if (pathname === "/api/ipc/official_admin_reconcile") {
        const events = process.env.TOKENTRACER_ADMIN_EVENTS;
        const spend = process.env.TOKENTRACER_ADMIN_SPEND;
        if (!events || !spend || !fs.existsSync(events) || !fs.existsSync(spend)) {
          sendError(
            res,
            404,
            "official_admin_reconcile stub: set TOKENTRACER_ADMIN_EVENTS + TOKENTRACER_ADMIN_SPEND (F14 optional; B-surface uses spend spending-align)",
          );
          return;
        }
        try {
          const { json, launcher } = await runSpend([
            "cursor",
            "official-reconcile",
            "--events",
            events,
            "--spend",
            spend,
          ]);
          sendJson(res, 200, json, {
            "X-TokenTracer-Source": "cli",
            "X-TokenTracer-Spend-Mode": launcher.mode,
          });
        } catch (e) {
          const message = e instanceof Error ? e.message : String(e);
          sendError(res, 503, message);
        }
        return;
      }

      sendError(res, 404, `unknown IPC path: ${pathname}`);
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      sendError(res, 503, message);
    }
  };
}

/** Vite plugin: wire middleware for `vite` and `vite preview`. */
export function spendDevBridgePlugin() {
  return {
    name: "tokentracer-spend-dev-bridge",
    configureServer(server) {
      server.middlewares.use(createSpendIpcMiddleware());
    },
    configurePreviewServer(server) {
      server.middlewares.use(createSpendIpcMiddleware());
    },
  };
}
