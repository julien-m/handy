// Fake Tauri IPC for the browser harness of the replacement rules editor (feature 002): saved
// rules, the shared check (no issue), Save at the expected revision and a refused import file,
// from a scripted backend that `tests/word-replacements-focus.spec.ts` drives through
// `window.harness`. A headless check of the real React code, not a native window proof. Imported
// first by the harness, so the fakes exist before any Tauri module is evaluated.

type Callback = (message: unknown) => void;
type Handler = (args: Record<string, unknown>) => unknown;

interface Rule {
  id: string;
  recognized: string;
  replacement: string;
}

/** Scripted backend: two saved rules. */
export const backend = {
  revision: 7,
  rules: [
    { id: "r1", recognized: "teh", replacement: "the" },
    { id: "r2", recognized: "handy app", replacement: "Handy" },
  ] as Rule[],
  saveCalls: 0,
  nextId: 3,
};

const callbacks = new Map<number, Callback>();
const listeners: Array<{ event: string; handler: number }> = [];
let nextCallback = 1;

const view = () => ({ revision: backend.revision, rules: [...backend.rules] });

// Save like `save_word_replacements`: new rules get an id, the revision moves on.
const save: Handler = (args) => {
  backend.saveCalls += 1;
  const drafts = args.rules as Array<Omit<Rule, "id"> & { id: string | null }>;
  backend.rules = drafts.map((rule) => ({
    ...rule,
    id: rule.id ?? `r${backend.nextId++}`,
  }));
  backend.revision += 1;
  return view();
};

// A file that is not UTF-8: the command rejects with the typed refusal.
const refuseImport: Handler = () => {
  throw {
    file_name: "notes-latin1.txt",
    error: {
      code: "invalid_value",
      message: "invalid encoding",
      field: "word_replacements",
      reason: "invalid_encoding",
      outcome: null,
      current_revision: backend.revision,
    },
  };
};

const handlers: Record<string, Handler> = {
  get_word_replacements: view,
  check_word_replacements: () => [],
  save_word_replacements: save,
  import_word_replacements_file: refuseImport,
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
