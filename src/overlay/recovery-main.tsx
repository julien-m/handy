// Entry of the recovery window (010), a separate nonactivating webview: theme set before
// render (shared localStorage), reconciled with the settings, then followed live.
import React from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import {
  applyTheme,
  getStoredTheme,
  syncThemeFromSettings,
} from "@/lib/utils/theme";
import type { Theme } from "@/bindings";
import "@/i18n";
import "./RecordingOverlay.css";
import { RecoveryWindow } from "./recovery-window";

applyTheme(getStoredTheme());
syncThemeFromSettings();
listen<Theme>("theme-changed", (event) => applyTheme(event.payload));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RecoveryWindow />
  </React.StrictMode>,
);
