const KEY = "calcine.onboarded";

/** Whether the welcome flow was completed or skipped on this machine. */
export function isOnboarded(): boolean {
  try {
    return localStorage.getItem(KEY) === "1";
  } catch {
    return true;
  }
}

export function markOnboarded() {
  try {
    localStorage.setItem(KEY, "1");
  } catch {
    // Storage unavailable: the welcome page simply shows again next time.
  }
}
