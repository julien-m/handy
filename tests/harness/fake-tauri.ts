// Fake Tauri IPC, page visibility and window visibility for the browser harness of the microphone
// priority editor (feature 001). It answers the commands the editor and the settings store call from a scripted
// backend that `tests/microphone-priority.spec.ts` drives through `window.harness`. This is a
// headless check of the real React code, not a native window proof.
// Imported first by the harness, so the fakes exist before any Tauri module is evaluated.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

interface Deferred {
  promise: Promise<void>;
  resolve: () => void;
}

const deferred = (): Deferred => {
  let resolve = () => undefined as void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};

/** Scripted backend state. */
export const backend = {
  revision: 3,
  priority: [
    "USB headset",
    "MacBook Pro Microphone",
    "Studio Display Microphone",
  ],
  devices: [
    "USB headset",
    "MacBook Pro Microphone",
    "Studio Display Microphone",
    "Dock",
  ],
  listingFails: false,
  statusCalls: 0,
  saveCalls: [] as Array<{ order: string[]; expectedRevision: unknown }>,
  saveGate: null as Deferred | null,
  saveFailure: null as Record<string, unknown> | null,
  /** The held write commits before it answers with `saveFailure` (an unknown outcome). */
  commitBeforeFailure: false,
  /** Answer of the window's `isVisible()`; the page visibility is set apart (`setVisible`). */
  windowVisible: true,
  windowVisibilityFails: false,
  windowVisibilityChecks: 0,
  settingsGate: null as Deferred | null,
};

const callbacks = new Map<number, Callback>();
const listeners: Array<{ event: string; handler: number }> = [];
let nextCallback = 1;

const emit = (event: string, payload: unknown): void => {
  for (const listener of listeners.filter((entry) => entry.event === event)) {
    callbacks.get(listener.handler)?.({ event, id: 0, payload });
  }
};

/** Commit an order as the backend would, then emit `settings-changed`. */
export const commit = (order: readonly string[]): void => {
  backend.priority = [...order];
  backend.revision += 1;
  emit("settings-changed", {
    revision: backend.revision,
    keys: ["microphone_priority"],
  });
};

const settingsView = () => ({
  revision: backend.revision,
  read_only: false,
  settings: {
    microphone_priority: [...backend.priority],
    always_on_microphone: false,
    clamshell_microphone: null,
    selected_output_device: null,
    audio_feedback: false,
    app_language: "en",
    bindings: {},
  },
});

const status = () => ({
  configured_order: [...backend.priority],
  availability: backend.priority.map((name) => ({
    name,
    status: backend.devices.includes(name) ? "available" : "unavailable",
  })),
  availability_observed_at: new Date().toISOString(),
  effective_input: {
    scope: "not_used_yet",
    name: null,
    reason: null,
    capture_generation: null,
  },
  warning: null,
});

const settingsError = (code: string) => ({
  code,
  message: code,
  field: "microphone_priority",
  reason: null,
  outcome: null,
  current_revision: backend.revision,
});

// Compare-and-set like `set_microphone_priority`: a stale revision is refused.
const savePriority: Handler = async (args) => {
  const order = args.order as string[];
  backend.saveCalls.push({ order, expectedRevision: args.expectedRevision });
  if (backend.saveGate) await backend.saveGate.promise;
  if (backend.saveFailure) {
    if (backend.commitBeforeFailure) commit(order);
    throw backend.saveFailure;
  }
  if (args.expectedRevision !== backend.revision) {
    throw settingsError("revision_conflict");
  }
  commit(order);
  return backend.revision;
};

// A failed listing rejects with a plain value, as a Tauri command error does.
const listMicrophones = () => {
  if (backend.listingFails) throw "listing failed";
  return backend.devices.map((name, index) => ({
    index: String(index),
    name,
    is_default: false,
  }));
};

const handlers: Record<string, Handler> = {
  get_settings_view: async () => {
    if (backend.settingsGate) await backend.settingsGate.promise;
    return settingsView();
  },
  get_default_settings: () => settingsView().settings,
  get_app_settings: () => settingsView().settings,
  check_custom_sounds: () => ({ start: false, stop: false }),
  is_update_checks_locked: () => false,
  is_laptop: () => false,
  get_available_microphones: () => listMicrophones(),
  get_microphone_status: () => {
    backend.statusCalls += 1;
    return status();
  },
  set_microphone_priority: savePriority,
  "plugin:window|is_visible": () => {
    backend.windowVisibilityChecks += 1;
    if (backend.windowVisibilityFails) throw "window unavailable";
    return backend.windowVisible;
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

// Page visibility the tests switch, as the webview does when the window is hidden in the tray.
let visibility: DocumentVisibilityState = "visible";
Object.defineProperty(document, "visibilityState", {
  configurable: true,
  get: () => visibility,
});
Object.defineProperty(document, "hidden", {
  configurable: true,
  get: () => visibility === "hidden",
});

/** Show or hide the page and fire `visibilitychange`. */
export const setVisible = (visible: boolean): void => {
  visibility = visible ? "visible" : "hidden";
  document.dispatchEvent(new Event("visibilitychange"));
};

/**
 * Hide or show the Tauri window without touching the page visibility, as WebView2 does on Windows
 * (the page stays "visible"). Showing it also focuses it, as `show_main_window` does; hiding it
 * emits nothing, the polling must find out by asking.
 */
export const setWindowShown = (shown: boolean): void => {
  backend.windowVisible = shown;
  if (shown) emit("tauri://focus", null);
};

/** Hold the next writes until `releaseSave`, which settles them with `failure` or success. */
export const holdSave = (): void => {
  backend.saveGate = deferred();
};

export const releaseSave = (failure: string | null): void => {
  backend.saveFailure = failure === null ? null : settingsError(failure);
  backend.commitBeforeFailure = false;
  backend.saveGate?.resolve();
  backend.saveGate = null;
};

/**
 * Settle the held write with an unknown outcome (code `unavailable`, outcome `unknown`, as the
 * backend's `unknown_outcome`), after committing it or not.
 */
export const releaseSaveUnknown = (committed: boolean): void => {
  backend.saveFailure = { ...settingsError("unavailable"), outcome: "unknown" };
  backend.commitBeforeFailure = committed;
  backend.saveGate?.resolve();
  backend.saveGate = null;
};

/** Hold the settings read (the editor stays in its loading state). */
export const holdSettings = (): void => {
  backend.settingsGate = deferred();
};

export const releaseSettings = (): void => {
  backend.settingsGate?.resolve();
  backend.settingsGate = null;
};
