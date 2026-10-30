// Browser harness of the "Show the bubble when no text field is focused" row (010): the real
// component and settings store against the fake IPC of `recovery-bubble-fake.ts`, for
// `tests/recovery-bubble.spec.ts`. A button before the row gives the keyboard a start.
import { backend } from "./recovery-bubble-fake";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { NoFieldBubbleToggle } from "@/components/settings/recovery-bubble-toggle";
import { useSettingsStore } from "@/stores/settingsStore";

const Row = () => (
  <div className="p-4 max-w-3xl" data-testid="output-settings-screen">
    <button type="button" data-testid="harness-start">
      start
    </button>
    <NoFieldBubbleToggle grouped={false} />
  </div>
);

const params = new URLSearchParams(window.location.search);
if (params.get("gate") === "closed") backend.noFieldBubble = null;

void useSettingsStore
  .getState()
  .refreshSettings()
  .then(() => {
    const element = document.getElementById("root");
    if (element) createRoot(element).render(<Row />);
  });

(window as unknown as Record<string, unknown>).harness = { backend };
