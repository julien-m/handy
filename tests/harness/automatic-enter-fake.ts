// Fake Tauri IPC for the browser harness of the Automatic Enter row (009): it answers the
// settings reads and the two write commands of the row from a scripted backend that
// `tests/pin-enter-ui.spec.ts` inspects through `window.harness`. A headless check of the real
// React code and settings store, not a native window proof. Imported first by the harness, so
// the fakes exist before any Tauri module is evaluated.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

/** Scripted backend: the open gate's settings (`automatic_enter` present). */
export const backend = {
  revision: 5,
  automaticEnter: "never",
  key: "enter",
  writes: [] as Array<{ command: string; value: unknown }>,
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
  automatic_enter: backend.automaticEnter,
  auto_submit_key: backend.key,
  app_language: "en",
  bindings: {},
  microphone_priority: [],
});

/** Commit one write as the settings service does, then emit `settings-changed`. */
const write =
  (command: string, field: "automaticEnter" | "key", key: string): Handler =>
  (args) => {
    const value = Object.values(args)[0];
    backend.writes.push({ command, value });
    backend[field] = String(value);
    backend.revision += 1;
    emit("settings-changed", { revision: backend.revision, keys: [key] });
    return null;
  };

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
  change_automatic_enter_setting: write(
    "change_automatic_enter_setting",
    "automaticEnter",
    "automatic_enter",
  ),
  change_auto_submit_key_setting: write(
    "change_auto_submit_key_setting",
    "key",
    "auto_submit_key",
  ),
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
