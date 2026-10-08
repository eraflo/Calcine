// Calcine site: code tabs, copy button, reveal on scroll, and the download
// buttons pointing at the newest installer.

document.documentElement.classList.add("js");

// Code tabs (arrow keys move between them, as in the ARIA tabs pattern).
const tabs = [...document.querySelectorAll('.tabs [role="tab"]')];
const select = (tab) => {
  for (const other of tabs) {
    const selected = other === tab;
    other.setAttribute("aria-selected", String(selected));
    other.tabIndex = selected ? 0 : -1;
    document.getElementById(other.getAttribute("aria-controls")).hidden = !selected;
  }
};
for (const tab of tabs) {
  tab.addEventListener("click", () => select(tab));
  tab.addEventListener("keydown", (event) => {
    const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
    if (!step) return;
    const next = tabs[(tabs.indexOf(tab) + step + tabs.length) % tabs.length];
    select(next);
    next.focus();
  });
}

const copy = document.querySelector(".tabs .copy");
copy?.addEventListener("click", async () => {
  const panel = document.querySelector('.code-card [role="tabpanel"]:not([hidden])');
  try {
    await navigator.clipboard.writeText(panel.textContent);
    copy.classList.add("done");
    setTimeout(() => copy.classList.remove("done"), 1500);
  } catch {
    // Clipboard unavailable (insecure context or denied): nothing to do.
  }
});

// Fade sections in as they scroll into view.
const revealed = document.querySelectorAll(
  ".units, .section-head, .grid, .row, .api, .api-shot, .layers, .download, .faq",
);
if ("IntersectionObserver" in window) {
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add("visible");
        observer.unobserve(entry.target);
      }
    },
    { rootMargin: "0px 0px -10% 0px" },
  );
  for (const element of revealed) {
    element.classList.add("reveal");
    observer.observe(element);
  }
}

// Link the download buttons to the newest installer: the latest stable
// release, or the newest beta while there is none. Without the GitHub API
// (offline, rate limit) they keep pointing at the releases page.
(async () => {
  try {
    const response = await fetch(
      "https://api.github.com/repos/eraflo/Calcine/releases?per_page=20",
      {
        headers: { Accept: "application/vnd.github+json" },
      },
    );
    if (!response.ok) return;
    const releases = (await response.json()).filter((release) => !release.draft);
    const installer = (release) =>
      release.assets.find((asset) => asset.name.endsWith("-setup.exe"));
    const withInstaller = releases.filter(installer);
    const release = withInstaller.find((r) => !r.prerelease) ?? withInstaller[0];
    if (!release) return;
    const asset = installer(release);
    const version = release.tag_name.replace(/^v/, "");
    const size = `${Math.round(asset.size / 1e6)} MB`;
    for (const link of document.querySelectorAll("[data-download]"))
      link.href = asset.browser_download_url;
    for (const label of document.querySelectorAll("[data-download-label]")) {
      label.textContent = `Download Calcine ${version}`;
    }
    for (const meta of document.querySelectorAll("[data-download-meta]")) {
      meta.textContent = `${release.prerelease ? "Beta · " : ""}Windows 11 ARM64 · ${size} · GenieX included`;
    }
  } catch {
    // Keep the releases page link.
  }
})();
