import { api, type Destination, type SaveMode, type Settings } from "../api";

/** App settings mirrored from the backend. */
class SettingsState {
  value = $state<Settings | null>(null);

  async load() {
    this.value = await api.getSettings();
    return this.value;
  }

  async setSaveMode(mode: SaveMode) {
    this.value = await api.setSaveMode(mode);
  }

  /** Where edits go without asking, or null when the user must be asked. */
  get defaultDestination(): Destination | null {
    const mode = this.value?.saveMode;
    return mode === "new" || mode === "replace" ? mode : null;
  }
}

export const settings = new SettingsState();
