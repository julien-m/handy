// Fake Tauri IPC for the browser harness of the recovery bubble (010): it answers the commands
// of the recovery window and of the no-text-field toggle from a scripted backend that
// `tests/recovery-bubble.spec.ts` drives through `window.harness`. A headless check of the real
// React code, not a native window proof (first click without activation, drag and VoiceOver
// are native checks). Imported first by the harnesses, so the fakes exist before any Tauri
// module is evaluated. `?os=windows` simulates another operating system.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

const os = new URLSearchParams(window.location.search).get("os") ?? "macos";

/** Scripted backend: the current projection, the setting and every call received. */
export const backend = {
  view: null as Record<string, unknown> | null,
  calls: [] as Array<{ command: string; args: Record<string, unknown> }>,
  revision: 4,
  noFieldBubble: true as boolean | null,
};

const callbacks = new Map<number, Callback>();
const listeners: Array<{ event: string; handler: number }> = [];
let nextCallback = 1;

/** Deliver a native event to the page. */
export const emit = (event: string, payload: unknown): void => {
  for (const listener of listeners.filter((entry) => entry.event === event)) {
    callbacks.get(listener.handler)?.({ event, id: 0, payload });
  }
};

const record =
  (command: string, answer: unknown = null): Handler =>
  (args) => {
    backend.calls.push({ command, args });
    return answer;
  };

const settings = () => ({
  app_language: "en",
  bindings: {},
  microphone_priority: [],
  ...(backend.noFieldBubble === null
    ? {}
    : { show_recovery_bubble_without_text_field: backend.noFieldBubble }),
});

const handlers: Record<string, Handler> = {
  recovery_bubble_view: () => backend.view,
  recovery_bubble_rendered: record("recovery_bubble_rendered"),
  recovery_bubble_copy: record("recovery_bubble_copy"),
  recovery_bubble_close: record("recovery_bubble_close"),
  recovery_bubble_begin_drag: record("recovery_bubble_begin_drag"),
  recovery_bubble_resize: record("recovery_bubble_resize"),
  get_settings_view: () => ({
    revision: backend.revision,
    read_only: false,
    settings: settings(),
  }),
  get_default_settings: () => settings(),
  get_app_settings: () => settings(),
  check_custom_sounds: () => ({ start: false, stop: false }),
  is_update_checks_locked: () => false,
  change_show_recovery_bubble_without_text_field_setting: (args) => {
    backend.calls.push({
      command: "change_show_recovery_bubble_without_text_field_setting",
      args,
    });
    backend.noFieldBubble = Boolean(args.enabled);
    backend.revision += 1;
    emit("settings-changed", {
      revision: backend.revision,
      keys: ["show_recovery_bubble_without_text_field"],
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
    currentWindow: { label: "recovery_bubble" },
    currentWebview: {
      windowLabel: "recovery_bubble",
      label: "recovery_bubble",
    },
  },
  convertFileSrc: (path: string) => path,
};
fakeWindow.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
  unregisterListener: () => undefined,
};
fakeWindow.__TAURI_OS_PLUGIN_INTERNALS__ = {
  platform: os,
  os_type: os,
  family: os === "windows" ? "windows" : "unix",
  version: "15",
  arch: "aarch64",
  eol: "\n",
  exe_extension: "",
};
