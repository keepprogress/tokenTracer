/// <reference types="vite/client" />

/** Tauri 2 withGlobalTauri — present inside tokentracer-host webview. */
interface TauriGlobal {
  core?: {
    invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
  };
  event?: {
    listen: (
      event: string,
      handler: (event: { payload: unknown }) => void,
    ) => Promise<() => void>;
  };
  invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
}

interface Window {
  __TAURI__?: TauriGlobal;
  __TAURI_INTERNALS__?: {
    invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
  };
}
