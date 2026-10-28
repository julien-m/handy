// Browser harness of the settings window frame (native session finding N4): the real
// `SettingsWindowLayout` around a section taller than the window whose last rows hold a visually
// hidden hint (`sr-only`, as the replacement and vocabulary editors use) and a focusable control.
// Served by the Vite dev server for `tests/settings-window.spec.ts`.
import React from "react";
import { createRoot } from "react-dom/client";
import "@/App.css";
import { SettingsWindowLayout } from "@/components/settings-window-layout";

const ROW_COUNT = 60;

const Section: React.FC = () => (
  <div className="max-w-3xl w-full flex flex-col gap-2">
    {Array.from({ length: ROW_COUNT }, (_, index) => (
      <div
        key={index}
        className="px-4 p-2 rounded-lg border border-mid-gray/20 text-sm"
      >
        {`Row ${index + 1}`}
      </div>
    ))}
    <span id="last-hint" className="sr-only">
      Hidden hint
    </span>
    <button
      type="button"
      data-testid="last-control"
      aria-describedby="last-hint"
    >
      Last control
    </button>
  </div>
);

const element = document.getElementById("root");
if (element !== null) {
  createRoot(element).render(
    <SettingsWindowLayout
      direction="ltr"
      sidebar={
        <nav
          data-testid="settings-window-sidebar"
          className="flex flex-col w-40 h-full border-e border-mid-gray/20 p-2"
        >
          Sidebar
        </nav>
      }
      footer={
        <footer
          data-testid="settings-window-footer"
          className="w-full border-t border-mid-gray/20 p-2 text-xs"
        >
          Footer
        </footer>
      }
    >
      <Section />
    </SettingsWindowLayout>,
  );
}
