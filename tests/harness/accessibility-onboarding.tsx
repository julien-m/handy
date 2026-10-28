// Browser harness of the "Permissions Required" screen (native session finding N10): the real
// onboarding step against the fake Tauri IPC of `accessibility-onboarding-fake.ts`, served by the
// Vite dev server for `tests/accessibility-onboarding.spec.ts`.
import { backend } from "./accessibility-onboarding-fake";
import React from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { AccessibilityOnboarding } from "@/components/onboarding";

const state = { completions: 0 };

const element = document.getElementById("root");
if (element !== null) {
  createRoot(element).render(
    <AccessibilityOnboarding onComplete={() => (state.completions += 1)} />,
  );
}

(window as unknown as Record<string, unknown>).harness = { backend, state };
