// Browser harness of the Automatic Enter row (009): the real component and settings store
// against the fake Tauri IPC of `automatic-enter-fake.ts`, served by the Vite dev server for
// `tests/pin-enter-ui.spec.ts`. A button before the row gives the keyboard a starting point.
import { backend } from "./automatic-enter-fake";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { AutomaticEnter } from "@/components/settings/automatic-enter";
import { useSettingsStore } from "@/stores/settingsStore";

const Row = () => (
  <div
    className="p-4 max-w-3xl"
    data-testid="output-settings-enter-options-screen"
  >
    <button type="button" data-testid="harness-start">
      start
    </button>
    <AutomaticEnter grouped={false} />
  </div>
);

void useSettingsStore
  .getState()
  .refreshSettings()
  .then(() => {
    const element = document.getElementById("root");
    if (element) createRoot(element).render(<Row />);
  });

(window as unknown as Record<string, unknown>).harness = { backend };
