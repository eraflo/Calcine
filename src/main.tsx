import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./styles/globals.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/app";

// A desktop app, not a page: the document never scrolls. Focusing an element
// outside the scrolling regions (a hidden radio) used to scroll it anyway,
// pushing the whole app out of the window, which then looked black.
window.addEventListener("scroll", () => {
  if (window.scrollX || window.scrollY) window.scrollTo(0, 0);
});

const root = document.getElementById("root");
if (!root) throw new Error("Missing #root element");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
