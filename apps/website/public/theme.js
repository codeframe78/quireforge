(() => {
  const root = document.documentElement;
  let stored = null;
  try {
    stored = localStorage.getItem("quireforge-theme");
  } catch {
    // Theme preference storage may be blocked; system preference still works.
  }
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  const effectiveTheme = () => {
    const selected = root.dataset.theme;
    return selected === "dark" || (selected === "system" && systemDark.matches)
      ? "dark"
      : "light";
  };

  const updateButton = () => {
    const button = document.querySelector("[data-theme-toggle]");
    if (!(button instanceof HTMLButtonElement)) return;
    const dark = effectiveTheme() === "dark";
    button.setAttribute("aria-pressed", String(dark));
    button.setAttribute(
      "aria-label",
      dark ? "Use light theme" : "Use dark theme",
    );
  };

  if (stored === "light" || stored === "dark") {
    root.dataset.theme = stored;
  }

  document.addEventListener("DOMContentLoaded", () => {
    updateButton();
    document
      .querySelector("[data-theme-toggle]")
      ?.addEventListener("click", () => {
        const next = effectiveTheme() === "dark" ? "light" : "dark";
        root.dataset.theme = next;
        try {
          localStorage.setItem("quireforge-theme", next);
        } catch {
          // The visual toggle remains useful when persistence is unavailable.
        }
        updateButton();
      });
  });

  systemDark.addEventListener("change", updateButton);
})();
