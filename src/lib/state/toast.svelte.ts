/** One short-lived message at the bottom of the window. */
class ToastState {
  message = $state<string | null>(null);
  #timer: ReturnType<typeof setTimeout> | undefined;

  show(message: string) {
    clearTimeout(this.#timer);
    this.message = message;
    this.#timer = setTimeout(() => (this.message = null), 3200);
  }
}

export const toast = new ToastState();
