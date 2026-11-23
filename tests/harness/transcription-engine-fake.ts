// Fake Tauri IPC for the browser harness of the transcription engine setting (024): it answers
// the settings store, the engine change and the ChatGPT sign-in status from a scripted backend
// that `tests/transcription-engine.spec.ts` drives through `window.harness`. A headless check of
// the real React code, not a native window proof. Imported first by the harness, so the fakes
// exist before any Tauri module is evaluated.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

/** Scripted backend: the stored engine, the sign-in status and every engine change. */
export const backend = {
  revision: 7,
  engine: "local",
  signIn: "signed_in" as "signed_in" | "signed_out",
  changes: [] as string[],
};

const callbacks = new Map<number, Callback>();
const listeners: Array<{ event: string; handler: number }> = [];
let nextCallback = 1;

const emit = (event: string, payload: unknown): void => {
  for (const listener of listeners.filter((entry) => entry.event === event)) {
    callbacks.get(listener.handler)?.({ event, id: 0, payload });
  }
};

const settings = () => ({
  app_language: "en",
  bindings: {},
  microphone_priority: [],
  transcription_engine: backend.engine,
});

const handlers: Record<string, Handler> = {
  get_settings_view: () => ({
    revision: backend.revision,
    read_only: false,
    settings: settings(),
  }),
  get_default_settings: () => settings(),
  get_app_settings: () => settings(),
  check_custom_sounds: () => ({ start: false, stop: false }),
  is_update_checks_locked: () => false,
  get_chatgpt_sign_in_status: () => backend.signIn,
  change_transcription_engine_setting: (args) => {
    const engine = String(
      (args.engine as { value?: unknown })?.value ?? args.engine,
    );
    // Mirrors the settings service: ChatGPT is refused while signed out (024 FR-009).
    if (engine === "chatgpt" && backend.signIn === "signed_out") {
      return Promise.reject({
        code: "validation_failed",
        message: "Setting value failed validation.",
        field: "transcription_engine",
        reason: "engine_sign_in_required",
        outcome: null,
        current_revision: null,
      });
    }
    backend.changes.push(engine);
    backend.engine = engine;
    backend.revision += 1;
    emit("settings-changed", {
      revision: backend.revision,
      keys: ["transcription_engine"],
    });
    return null;
  },
  "plugin:os|locale": () => "en-US",
  "plugin:event|listen": (args) => {
    listeners.push({
      event: String(args.event),
      handler: Number(args.handler),
    });
    return listeners.length;
  },
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
