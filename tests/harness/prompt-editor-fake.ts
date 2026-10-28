// Fake Tauri IPC for the browser harness of the prompt editor (feature 005): one saved prompt,
// the settings view the editor reads, and Save (`update_post_process_prompt`) committing a new
// revision, from a scripted backend that `tests/prompt-editor-focus.spec.ts` inspects through
// `window.harness`. A headless check of the real React code, not a native window proof.
// Imported first by the harness, so the fakes exist before any Tauri module is evaluated.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

/** Scripted backend: one saved prompt, no provider configured. */
export const backend = {
  revision: 11,
  prompt: { id: "p1", name: "Meeting notes", text: "Summarize ${output}" },
  updates: 0,
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
  post_process_enabled: true,
  post_process_providers: [],
  post_process_provider_id: "",
  post_process_models: {},
  post_process_api_keys: {},
});

const library = () => ({
  default_prompt_id: backend.prompt.id,
  post_process_enabled: true,
  prompts: [
    {
      id: backend.prompt.id,
      name: backend.prompt.name,
      prompt: backend.prompt.text,
      provider_override: null,
      starter: false,
      issues: [],
      shortcuts: [],
      readiness: {
        provider_id: null,
        model: null,
        is_override: false,
        ready: false,
        missing: null,
      },
    },
  ],
});

// Save like `update_post_process_prompt`: the prompt changes, the revision moves on.
const update: Handler = (args) => {
  backend.updates += 1;
  backend.prompt = {
    ...backend.prompt,
    name: String(args.name),
    text: String(args.prompt),
  };
  backend.revision += 1;
  emit("settings-changed", {
    revision: backend.revision,
    keys: ["post_process_prompts"],
  });
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
  get_prompt_library: library,
  update_post_process_prompt: update,
  get_shortcut_conflicts: () => [],
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
