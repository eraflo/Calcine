import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./styles/globals.css";

import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

// A desktop app, not a page: the document never scrolls. Focusing an element
// outside the scrolling regions (a hidden radio) used to scroll it anyway,
// pushing the whole app out of the window, which then looked black.
window.addEventListener("scroll", () => {
  if (window.scrollX || window.scrollY) window.scrollTo(0, 0);
});

const root = document.getElementById("root");
if (!root) throw new Error("Missing #root element");

// The quick question window loads the same page and shows only its own UI.
const Root =
  isTauri() && getCurrentWindow().label === "quick"
    ? (await import("./features/quick-ask/quick-ask-app")).QuickAskApp
    : (await import("./app/app")).App;

createRoot(root).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
