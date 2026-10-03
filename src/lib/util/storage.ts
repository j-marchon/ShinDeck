/** Tiny typed wrapper around localStorage for UI preferences (never critical data). */
export function storedValue<T>(key: string, fallback: T) {
  const fullKey = `shindeck:${key}`;
  return {
    get(): T {
      try {
        const raw = localStorage.getItem(fullKey);
        if (raw == null) return fallback;
        const parsed = JSON.parse(raw);
        // Merge objects so newly added preference fields get their defaults.
        return typeof fallback === "object" && fallback !== null ? { ...fallback, ...parsed } : parsed;
      } catch {
        return fallback;
      }
    },
    set(value: T) {
      try {
        localStorage.setItem(fullKey, JSON.stringify(value));
      } catch {
        /* storage unavailable: preference simply isn't remembered */
      }
    },
  };
}
