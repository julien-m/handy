// Browser harness of the recovery window (010): the real `RecoveryWindow` with its hook and
// styles against the fake IPC of `recovery-bubble-fake.ts`, served by the Vite dev server for
// `tests/recovery-bubble.spec.ts`. A button before the bubble gives the keyboard a start.
import { backend, emit } from "./recovery-bubble-fake";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/overlay/RecordingOverlay.css";
import { RecoveryWindow } from "@/overlay/recovery-window";

const element = document.getElementById("root");
if (element) {
  createRoot(element).render(
    <div className="dark p-6" data-testid="harness-desktop">
      <button type="button" data-testid="harness-start">
        start
      </button>
      <RecoveryWindow />
    </div>,
  );
}

(window as unknown as Record<string, unknown>).harness = { backend, emit };
