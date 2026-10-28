// Fake Tauri IPC for the browser harness of the "Permissions Required" screen (native session
// finding N10): macOS, the microphone already granted, the accessibility permission off until
// the test grants it, and the opener recording the URLs it is asked to open. A headless check of
// the real React code, not a native window proof. Imported first by the harness.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

/** Scripted backend. */
export const backend = {
  accessibility: false,
  accessibilityChecks: 0,
  accessibilityRequests: 0,
  openedUrls: [] as string[],
};

const callbacks = new Map<number, Callback>();
let nextCallback = 1;

const handlers: Record<string, Handler> = {
  "plugin:macos-permissions|check_accessibility_permission": () => {
    backend.accessibilityChecks += 1;
    return backend.accessibility;
  },
  "plugin:macos-permissions|check_microphone_permission": () => true,
  "plugin:macos-permissions|request_accessibility_permission": () => {
    backend.accessibilityRequests += 1;
    return null;
  },
  "plugin:opener|open_url": (args) => {
    backend.openedUrls.push(String(args.url));
    return null;
  },
  get_available_microphones: () => [],
  get_available_output_devices: () => [],
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
