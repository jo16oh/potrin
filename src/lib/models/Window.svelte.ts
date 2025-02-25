let hasFocus = $state(document.hasFocus());
window.addEventListener("focus", () => (hasFocus = true));
window.addEventListener("blur", () => (hasFocus = false));

export const Window = {
  hasFocus() {
    return hasFocus;
  }
}

