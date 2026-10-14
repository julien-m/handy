// Browser harness of the microphone priority editor (feature 001): the real editor, settings
// store and status hook against the fake Tauri IPC of `fake-tauri.ts`, served by the Vite dev
// server for `tests/microphone-priority.spec.ts`. Query flags: `holdSettings` keeps the settings
// read pending (loading state).
import {
  backend,
  commit,
  holdSave,
  holdSettings,
  releaseSave,
  releaseSaveUnknown,
  releaseSettings,
  setVisible,
  setWindowShown,
} from "./fake-tauri";
import React from "react";
import { createRoot, type Root } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import {
  MicrophonePriorityRows,
  MicrophonePrioritySettings,
} from "@/components/settings/microphone-priority/microphone-priority-settings";
import { useSettingsStore } from "@/stores/settingsStore";

const Editor: React.FC = () => (
  <div className="p-4 max-w-3xl">
    <MicrophonePrioritySettings>
      <div className="border border-mid-gray/20 rounded-lg">
        <MicrophonePriorityRows />
      </div>
    </MicrophonePrioritySettings>
  </div>
);

let root: Root | null = null;

const mount = (): void => {
  const element = document.getElementById("root");
  if (element === null || root !== null) return;
  root = createRoot(element);
  root.render(<Editor />);
};

const unmount = (): void => {
  root?.unmount();
  root = null;
};

if (new URLSearchParams(window.location.search).has("holdSettings")) {
  holdSettings();
}
void useSettingsStore.getState().refreshAudioDevices();
mount();

(window as unknown as Record<string, unknown>).harness = {
  backend,
  commitElsewhere: commit,
  holdSave,
  releaseSave,
  releaseSaveUnknown,
  releaseSettings,
  setVisible,
  setWindowShown,
  mount,
  unmount,
};
