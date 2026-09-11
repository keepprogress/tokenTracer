import { defineConfig } from "vite";

// Dev server is debug-only localhost; production embed is Tauri webview (no host needed).
export default defineConfig({
  root: ".",
  base: "./",
  server: {
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
  },
});
