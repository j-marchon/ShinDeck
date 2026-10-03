import type { Backend } from "./types";

export type * from "./types";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// In a plain browser there is no transparent window behind the page.
if (!inTauri && typeof document !== "undefined") document.documentElement.style.background = "#000";

/**
 * The active backend. Outside Tauri (plain `npm run dev` in a browser) a mock
 * with generated data is used so the UI can be worked on without the shell.
 * Each implementation lives in its own chunk, so the app never loads the mock.
 */
export const api: Backend = inTauri
  ? (await import("./tauri")).tauriBackend
  : (await import("./mock")).mockBackend;
