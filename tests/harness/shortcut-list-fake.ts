// Fake Tauri IPC for the browser harness of a dictation shortcut list (feature 004) with the
// HandyKeys recorder: the macOS Secure Input refusal of a recording start is scripted, so the
// "Add shortcut" draft opens blocked (native session finding N8, review). A headless check of
// the real React code, not a native window proof. Imported first by the harness.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

/** Scripted backend: one dictation shortcut, recordings refused by Secure Input. */
export const backend = {
  recordingStarts: 0,
  refuseRecording: true,
};

const callbacks = new Map<number, Callback>();
let nextCallback = 1;

const binding = (id: string, shortcut: string) => ({
  id,
  name: id,
  description: `${id} description`,
  default_binding: shortcut,
  current_binding: shortcut,
  additional_bindings: [],
});

const settings = () => ({
  app_language: "en",
  keyboard_implementation: "handy_keys",
  microphone_priority: [],
  bindings: { transcribe: binding("transcribe", "option+space") },
});

const startRecording: Handler = () => {
  backend.recordingStarts += 1;
  if (backend.refuseRecording) throw "secure-input-active";
  return null;
};

const handlers: Record<string, Handler> = {
  get_settings_view: () => ({
    revision: 4,
    read_only: false,
    settings: settings(),
  }),
  get_default_settings: () => settings(),
  get_app_settings: () => settings(),
  check_custom_sounds: () => ({ start: false, stop: false }),
  is_update_checks_locked: () => false,
  get_shortcut_conflicts: () => [],
  start_handy_keys_recording: startRecording,
  "plugin:os|locale": () => "en-US",
  "plugin:event|listen": () => 1,
};

const fakeWindow = window as unknown as Record<string, unknown>;
fakeWindow.__TAURI_INTERNALS__ = {
  transformCallback: (callback: Callback) => {
    const id = nextCallback++;
    callbacks.set(id, callback);
    return id;
  },
  unregisterCallback: (id: number) => callbacks.delete(id),
  invoke: async (command: string, args?: Record<string, unknown>) =>
    handlers[command] ? handlers[command](args ?? {}) : null,
  metadata: {
    currentWindow: { label: "main" },
    currentWebview: { windowLabel: "main", label: "main" },
  },
  convertFileSrc: (path: string) => path,
};
fakeWindow.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
  unregisterListener: () => undefined,
};
fakeWindow.__TAURI_OS_PLUGIN_INTERNALS__ = {
  platform: "macos",
  os_type: "macos",
  family: "unix",
  version: "15",
  arch: "aarch64",
  eol: "\n",
  exe_extension: "",
};
