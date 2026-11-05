// Entry of the Quick Add window (018): theme set before render (shared localStorage), reconciled
// with the settings, then followed live, like the recovery window.
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
import "@/App.css";
import { QuickAddWindow } from "./quick-add-window";
import { parseKind } from "./quick-add-model";

applyTheme(getStoredTheme());
syncThemeFromSettings();
listen<Theme>("theme-changed", (event) => applyTheme(event.payload));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QuickAddWindow initialKind={parseKind(window.location.search)} />
  </React.StrictMode>,
);
