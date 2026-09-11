import { defineConfig } from "vite";
import { spendDevBridgePlugin } from "./scripts/spend-dev-bridge.mjs";

// Dev/preview: localhost + /api/ipc → real `spend` CLI (shell-host command names).
// Production Tauri embed has no Vite host; api.ts falls back to bundled fixtures
// until Tauri `invoke` lands.
export default defineConfig({
  root: ".",
  base: "./",
  plugins: [spendDevBridgePlugin()],
  server: {
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
    port: 4173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
  },
});
