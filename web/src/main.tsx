import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { AppShell } from "./app/AppShell";
import { FlauzAppProvider } from "./state/app";
import "./styles.css";

const rootElement = document.getElementById("flauz-root");
if (rootElement === null) {
  throw new Error("the #flauz-root mount point is missing from index.html");
}

createRoot(rootElement).render(
  <StrictMode>
    <FlauzAppProvider>
      <AppShell />
    </FlauzAppProvider>
  </StrictMode>,
);
