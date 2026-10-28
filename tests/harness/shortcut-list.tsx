// Browser harness of the dictation shortcut list (feature 004): the real list and HandyKeys
// recorder against the fake Tauri IPC of `shortcut-list-fake.ts`, served by the Vite dev server
// for `tests/shortcut-list-draft.spec.ts`.
import { backend } from "./shortcut-list-fake";
import React from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { ShortcutList } from "@/components/settings/shortcut-list";
import { useSettingsStore } from "@/stores/settingsStore";

void useSettingsStore.getState().initialize();
const element = document.getElementById("root");
if (element !== null) {
  createRoot(element).render(
    <div className="p-4 max-w-3xl">
      <ShortcutList actionId="transcribe" />
    </div>,
  );
}

(window as unknown as Record<string, unknown>).harness = { backend };
