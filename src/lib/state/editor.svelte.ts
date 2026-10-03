import { api, type Clip, type Destination, type EditSpec } from "../api";
import { settings } from "./settings.svelte";

interface Prompt {
  /** Pre-ticks "remember my choice" the very first time. */
  remember: boolean;
  resolve: (destination: Destination | null) => void;
}

/** One running export at a time, plus the "new clip or replace?" prompt. */
class EditorState {
  running = $state(false);
  progress = $state(0);
  label = $state("");
  error = $state<string | null>(null);
  /** Short-lived success message. */
  notice = $state<string | null>(null);
  prompt = $state<Prompt | null>(null);

  #noticeTimer: ReturnType<typeof setTimeout> | undefined;

  /** The saved default, or asks the user. null = cancelled. */
  destination(): Promise<Destination | null> {
    const saved = settings.defaultDestination;
    if (saved) return Promise.resolve(saved);
    return new Promise((resolve) => {
      this.prompt = { remember: settings.value?.saveMode == null, resolve };
    });
  }

  async answer(choice: Destination | null, remember: boolean) {
    const prompt = this.prompt;
    this.prompt = null;
    if (choice) {
      // First answer either becomes the default or switches to "always ask".
      if (remember) await settings.setSaveMode(choice);
      else if (settings.value?.saveMode == null) await settings.setSaveMode("ask");
    }
    prompt?.resolve(choice);
  }

  async export(clip: Clip, spec: EditSpec, destination: Destination, label: string): Promise<Clip | null> {
    this.running = true;
    this.progress = 0;
    this.label = label;
    this.error = null;
    const unlisten = await api.onExportProgress((p) => (this.progress = p));
    try {
      const result = await api.exportClip(clip.id, spec, destination);
      this.#notify(destination === "new" ? "Saved as a new clip" : "Clip updated");
      return result;
    } catch (e) {
      const message = String(e).replace(/^Error: /, "");
      if (message !== "Cancelled") this.error = message;
      return null;
    } finally {
      unlisten();
      this.running = false;
    }
  }

  cancel() {
    api.cancelExport();
  }

  #notify(message: string) {
    clearTimeout(this.#noticeTimer);
    this.notice = message;
    this.#noticeTimer = setTimeout(() => (this.notice = null), 2600);
  }
}

export const editor = new EditorState();
