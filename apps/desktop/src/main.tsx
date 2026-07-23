import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { AppCrashBoundary } from "./AppCrashBoundary";
import { AppLoader } from "./AppLoader";

import "./styles.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("QuireForge root element is missing");
}

createRoot(root).render(
  <StrictMode>
    <AppCrashBoundary>
      <AppLoader />
    </AppCrashBoundary>
  </StrictMode>,
);
