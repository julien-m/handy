/**
 * Settings store against stubbed commands: refresh responses are ordered by revision, and a failed
 * command never restores a value older than what the backend committed (event/read ordering).
 * Run with `bun run test:settings-store` (Bun's test runner, for its module mocking).
 */
import { beforeEach, describe, expect, mock, test } from "bun:test";
import type { SettingsCommandError, SettingsView } from "@/bindings";

type Deferred<T> = { promise: Promise<T>; resolve: (value: T) => void };
const deferred = <T>(): Deferred<T> => {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};
interface ModeReply {
  status: "ok" | "error";
  data?: null;
  error?: SettingsCommandError;
}
const view = (revision: number, alwaysOn: boolean): SettingsView =>
  ({
    revision,
    // Only the field under test matters; the store spreads the rest unchanged.
    settings: { always_on_microphone: alwaysOn },
    read_only: false,
  }) as unknown as SettingsView;

// Each scenario queues the replies the stubbed commands return, in call order.
let views: Array<Promise<SettingsView>> = [];
let modeReplies: Array<Promise<ModeReply>> = [];
let returnReplies: Array<Promise<ModeReply>> = [];
const returnCalls: unknown[] = [];
const commands = {
  getSettingsView: () => views.shift() ?? Promise.resolve(view(0, false)),
  updateMicrophoneMode: () =>
    modeReplies.shift() ?? Promise.resolve({ status: "ok", data: null }),
  changeReturnToDictationAppSetting: (enabled: unknown) => {
    returnCalls.push(enabled);
    return (
      returnReplies.shift() ?? Promise.resolve({ status: "ok", data: null })
    );
  },
};
// The real modules need Tauri IPC and Vite's import.meta.glob; the store only needs these.
mock.module("@/bindings", () => ({ commands }));
mock.module("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
const toastErrors: string[] = [];
mock.module("sonner", () => ({
  toast: {
    error: (message: string) => toastErrors.push(message),
    warning: () => undefined,
  },
}));
mock.module("@/i18n", () => ({ default: { t: (key: string) => key } }));
// Dynamic import: the store must load after the mocks above are registered.
const { useSettingsStore } = await import("./settingsStore");

const failure = (error: Partial<SettingsCommandError>): ModeReply => ({
  status: "error",
  error: {
    code: "unavailable",
    message: "Settings operation is unavailable.",
    field: "always_on_microphone",
    reason: null,
    outcome: null,
    current_revision: null,
    ...error,
  },
});
const committed = failure({
  code: "internal_error",
  message: "Settings operation failed.",
  outcome: "committed",
  current_revision: 5,
});
const refused = failure({});
const store = () => useSettingsStore.getState();
const alwaysOn = () => store().settings?.always_on_microphone;

describe("settings store ordering", () => {
  beforeEach(async () => {
    useSettingsStore.setState({ settings: null, revision: null });
    views = [Promise.resolve(view(4, false))];
    modeReplies = [];
    await store().refreshSettings();
  });

  test("an older refresh answering after a newer one is discarded", async () => {
    const older = deferred<SettingsView>();
    views = [older.promise, Promise.resolve(view(6, true))];
    const first = store().refreshSettings();
    await store().refreshSettings();
    older.resolve(view(5, false));
    await first;
    expect(store().revision).toBe(6);
    expect(alwaysOn()).toBe(true);
  });

  test("a committed value whose effect failed is reread, not rolled back", async () => {
    const reply = deferred<ModeReply>();
    modeReplies = [reply.promise];
    // The event refresh (revision 5) lands before the command's error reply.
    views = [Promise.resolve(view(5, true)), Promise.resolve(view(5, true))];
    const update = store().updateSetting("always_on_microphone", true);
    await store().refreshSettings();
    reply.resolve(committed);
    await update;
    expect(alwaysOn()).toBe(true);
    expect(store().revision).toBe(5);
  });

  test("a refusal with nothing newer applied rolls the optimistic value back", async () => {
    modeReplies = [Promise.resolve(refused)];
    await store().updateSetting("always_on_microphone", true);
    expect(alwaysOn()).toBe(false);
    expect(store().revision).toBe(4);
  });

  test("a refusal after another writer committed shows that writer's value", async () => {
    const reply = deferred<ModeReply>();
    modeReplies = [reply.promise];
    // Another surface (the CLI) committed revision 7 while the UI command was in flight.
    views = [Promise.resolve(view(7, true)), Promise.resolve(view(7, true))];
    const update = store().updateSetting("always_on_microphone", false);
    await store().refreshSettings();
    reply.resolve(refused);
    await update;
    expect(alwaysOn()).toBe(true);
    expect(store().revision).toBe(7);
  });
});

// 006: the settings UI write of `return_to_dictation_app` and its typed refusal.
describe("return to the dictation app", () => {
  const returnView = (revision: number, enabled: boolean): SettingsView =>
    ({
      revision,
      settings: { return_to_dictation_app: enabled },
      read_only: false,
    }) as unknown as SettingsView;
  const returnTo = () => store().settings?.return_to_dictation_app;

  beforeEach(async () => {
    useSettingsStore.setState({ settings: null, revision: null });
    views = [Promise.resolve(returnView(4, true))];
    returnReplies = [];
    returnCalls.length = 0;
    toastErrors.length = 0;
    await store().refreshSettings();
  });

  test("a change is sent to the typed command and kept", async () => {
    expect(await store().updateSetting("return_to_dictation_app", false)).toBe(
      true,
    );
    expect(returnCalls.join()).toBe("false");
    expect(returnTo()).toBe(false);
    expect(toastErrors.length).toBe(0);
  });

  test("an unsupported platform refusal is shown with its reason and rolled back", async () => {
    returnReplies = [
      Promise.resolve(
        failure({
          code: "validation_failed",
          message: "Setting value is invalid.",
          field: "return_to_dictation_app",
          reason: "unsupported_platform",
        }),
      ),
    ];
    expect(await store().updateSetting("return_to_dictation_app", false)).toBe(
      false,
    );
    expect(returnTo()).toBe(true);
    expect(toastErrors.join()).toBe("errors.settings.unsupportedPlatform");
  });

  test("a busy instance is shown as unavailable and rolled back", async () => {
    returnReplies = [
      Promise.resolve(failure({ field: "return_to_dictation_app" })),
    ];
    await store().updateSetting("return_to_dictation_app", false);
    expect(returnTo()).toBe(true);
    expect(toastErrors.join()).toBe("errors.settings.unavailable");
  });
});
