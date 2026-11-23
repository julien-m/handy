// Browser harness of the transcription engine tabs (024): the real tabs and settings store
// against the fake IPC of `transcription-engine-fake.ts`, with a placeholder for the model list,
// for `tests/transcription-engine.spec.ts`. `?engine=chatgpt&signIn=signed_out` presets the backend.
import { backend } from "./transcription-engine-fake";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { TranscriptionEngineTabs } from "@/components/settings/transcription-engine";
import { useSettingsStore } from "@/stores/settingsStore";

const params = new URLSearchParams(window.location.search);
backend.engine = params.get("engine") ?? backend.engine;
if (params.get("signIn") === "signed_out") backend.signIn = "signed_out";

const Screen = () => (
  <div className="p-4 max-w-3xl" data-testid="transcription-engine-screen">
    <TranscriptionEngineTabs>
      <div
        className="rounded-xl border-2 border-mid-gray/20 px-4 py-3 text-sm"
        data-testid="harness-model-list"
      >
        Local model list
      </div>
    </TranscriptionEngineTabs>
  </div>
);

void useSettingsStore
  .getState()
  .refreshSettings()
  .then(() => {
    const element = document.getElementById("root");
    if (element) createRoot(element).render(<Screen />);
  });

(window as unknown as Record<string, unknown>).harness = { backend };
