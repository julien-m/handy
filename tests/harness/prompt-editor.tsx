// Browser harness of the prompt editor (feature 005): the real editor of the saved prompt, fed by
// the real library hook and settings store, against the fake Tauri IPC of
// `prompt-editor-fake.ts`; served by the Vite dev server for `tests/prompt-editor-focus.spec.ts`.
import { backend } from "./prompt-editor-fake";
import React from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { PromptEditor } from "@/components/settings/post-processing/prompt-editor";
import { usePromptLibrary } from "@/components/settings/post-processing/use-prompt-library";
import { useSettingsStore } from "@/stores/settingsStore";

// `?webkitFocus`: like WebKit, no focusout reaches the page when the focused element is removed
// or disabled, so only the section's DOM observer can notice the lost focus.
if (new URLSearchParams(window.location.search).has("webkitFocus")) {
  window.addEventListener(
    "focusout",
    (event) => event.stopImmediatePropagation(),
    true,
  );
}

const Editor: React.FC = () => {
  const libraryState = usePromptLibrary();
  if (libraryState.library === null) return null;
  return (
    <PromptEditor
      target={{ mode: "edit", id: backend.prompt.id }}
      libraryState={libraryState}
      onCreated={() => undefined}
      onClose={() => undefined}
    />
  );
};

void useSettingsStore.getState().initialize();
const element = document.getElementById("root");
if (element !== null) createRoot(element).render(<Editor />);

(window as unknown as Record<string, unknown>).harness = { backend };
