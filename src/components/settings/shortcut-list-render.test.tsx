/**
 * Rendered states of the shortcut lists (004 shortcut-settings, -conflict and -limit screens):
 * contract test ids, the Add control state, the hints, the Cancel row and the alerts.
 * Run with `bun run test:shortcut-states`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ShortcutBinding } from "@/bindings";

// Translations render as "key params" so the markup shows which text is used.
mock.module("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: unknown) =>
      typeof params === "object" ? `${key} ${JSON.stringify(params)}` : key,
  }),
}));
mock.module("@tauri-apps/plugin-os", () => ({ type: () => "macos" }));
mock.module("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
mock.module("@/bindings", () => ({
  commands: { getShortcutConflicts: async () => [] },
}));

const binding = (id: string, shortcuts: string[]): ShortcutBinding => ({
  id,
  name: id,
  description: `${id} description`,
  default_binding: shortcuts[0],
  current_binding: shortcuts[0],
  additional_bindings: shortcuts.slice(1),
});
const bindings: Record<string, ShortcutBinding> = {
  transcribe: binding("transcribe", ["alt+space", "ctrl+alt+d", "f5", "f6"]),
  transcribe_with_post_process: binding("transcribe_with_post_process", [
    "alt+shift+space",
  ]),
  cancel: binding("cancel", ["escape"]),
};
mock.module("../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) =>
      key === "bindings"
        ? bindings
        : key === "keyboard_implementation"
          ? "tauri"
          : undefined,
    editBindingShortcuts: async () => null,
    refusedShortcuts: [],
    isUpdating: () => false,
    isLoading: false,
  }),
}));

const { ShortcutList } = await import("./shortcut-list");
const { LegacyConflictAlert, SystemRefusalAlert } = await import(
  "./shortcut-list-alerts"
);
const { AddRow, DraftRow, ListMessages, ShortcutRow } = await import(
  "./shortcut-list-rows"
);
const { CLOSED_DRAFT, draftReducer, listControls } = await import(
  "./shortcut-draft-model"
);
const { actOnRecordRequest } = await import("./shortcut-chip");
const {
  NO_RECORD_REQUEST,
  dropUnlistedRequest,
  markRequestHandled,
  requestFor,
  requestRecording,
} = await import("./record-request-model");
const { describeStartFailure, shortcutTestIds } = await import(
  "./shortcut-list-feedback"
);

const count = (markup: string, text: string): number =>
  markup.split(text).length - 1;
const names = {
  actionName: (id: string) => `name:${id}`,
  shortcutLabel: (shortcut: string) => `label:${shortcut}`,
};
const noop = async () => undefined;

describe("dictation list at the limit (shortcut-settings-limit)", () => {
  const markup = renderToStaticMarkup(
    <ShortcutList actionId="transcribe" grouped />,
  );

  test("row, title and every shortcut carry the contract ids", () => {
    expect(
      markup.includes('data-testid="shortcut-settings-limit-dictation"'),
    ).toBe(true);
    expect(
      markup.includes('data-testid="shortcut-settings-limit-dictation-title"'),
    ).toBe(true);
    expect(
      count(markup, 'data-testid="shortcut-settings-limit-dictation-shortcut"'),
    ).toBe(4);
    expect(
      count(
        markup,
        'data-testid="shortcut-settings-limit-dictation-shortcut-remove"',
      ),
    ).toBe(4);
  });

  test("every shortcut carries its instance id (shortcut-{keys})", () => {
    expect(count(markup, 'data-instance-id="shortcut-')).toBe(4);
    expect(markup.includes("shortcut-settings-dictation")).toBe(false);
  });

  test("Add is disabled and the limit hint explains why", () => {
    expect(
      markup.includes(
        'data-testid="shortcut-settings-limit-dictation-add" data-state="disabled"',
      ),
    ).toBe(true);
    expect(
      markup.includes('data-testid="shortcut-settings-limit-dictation-limit"'),
    ).toBe(true);
    expect(
      markup.includes("limitHint {&quot;count&quot;:4,&quot;limit&quot;:4}"),
    ).toBe(true);
  });

  test("the list starts idle with no draft recorder", () => {
    expect(
      markup.includes(
        'data-testid="shortcut-settings-limit-dictation-list" data-state="idle"',
      ),
    ).toBe(true);
    expect(markup.includes("shortcut-settings-limit-dictation-recording")).toBe(
      false,
    );
  });
});

describe("AI dictation with one shortcut (shortcut-settings)", () => {
  const markup = renderToStaticMarkup(
    <ShortcutList actionId="transcribe_with_post_process" grouped />,
  );

  test("Add is enabled and the last shortcut cannot be removed", () => {
    expect(
      markup.includes(
        'data-testid="shortcut-settings-ai-dictation-add" data-state="enabled"',
      ),
    ).toBe(true);
    expect(markup.includes('aria-disabled="true"')).toBe(true);
    expect(
      markup.includes('data-testid="shortcut-settings-ai-dictation-last"'),
    ).toBe(true);
  });
});

describe("Cancel row", () => {
  const markup = renderToStaticMarkup(
    <ShortcutList actionId="cancel" grouped single />,
  );

  test("one shortcut and Reset, no Add, no Remove, no hint", () => {
    expect(
      markup.includes('data-testid="shortcut-settings-cancel-shortcut"'),
    ).toBe(true);
    expect(
      markup.includes('data-testid="shortcut-settings-cancel-reset"'),
    ).toBe(true);
    expect(markup.includes("shortcut-settings-cancel-add")).toBe(false);
    expect(markup.includes("shortcut-settings-cancel-shortcut-remove")).toBe(
      false,
    );
    expect(markup.includes("shortcut-settings-cancel-last")).toBe(false);
  });
});

describe("alerts (shortcut-settings-conflict)", () => {
  test("an inherited conflict, on Cancel too, offers Change", () => {
    const warning = {
      shortcut: "escape",
      title: {
        key: "legacyOverlap",
        params: { shortcut: "Esc", action: "Dictation" },
      },
    };
    const markup = renderToStaticMarkup(
      <LegacyConflictAlert
        warning={warning}
        ids={shortcutTestIds("cancel", "shortcut-settings-conflict")}
        busy={false}
        onChange={() => undefined}
      />,
    );
    expect(
      markup.includes('data-testid="shortcut-settings-conflict-cancel-legacy"'),
    ).toBe(true);
    expect(
      markup.includes(
        'data-testid="shortcut-settings-conflict-cancel-legacy-title"',
      ),
    ).toBe(true);
    expect(
      markup.includes('data-testid="shortcut-settings-conflict-cancel-change"'),
    ).toBe(true);
    expect(markup.includes("legacyBody")).toBe(true);
  });

  test("a system refusal names the system and offers Try again", () => {
    const refusal = {
      shortcut: "cmd+shift+5",
      reason: "HotKey already registered",
      edit: { kind: "add", shortcut: "cmd+shift+5" } as const,
    };
    const markup = renderToStaticMarkup(
      <SystemRefusalAlert
        refusal={refusal}
        ids={shortcutTestIds("transcribe", "shortcut-settings-conflict")}
        names={names}
        busy={false}
        onRetry={() => undefined}
      />,
    );
    expect(markup.includes('role="alert"')).toBe(true);
    expect(
      markup.includes(
        'data-testid="shortcut-settings-conflict-dictation-os-title"',
      ),
    ).toBe(true);
    expect(markup.includes("&quot;system&quot;:&quot;macOS&quot;")).toBe(true);
    expect(
      markup.includes(
        'data-testid="shortcut-settings-conflict-dictation-os-retry"',
      ),
    ).toBe(true);
    expect(markup.includes("HotKey already registered")).toBe(true);
  });
});

describe("draft recorder", () => {
  const draft = (
    phase: "recording" | "blocked" | "invalid",
    shortcut: string | null,
  ) => ({
    state: { phase, shortcut, refusal: null, startFailure: null },
    recordRequest: 0,
    open: () => undefined,
    capture: noop,
    stop: () => undefined,
    commit: noop,
    discard: () => undefined,
    recorder: {
      requestHandled: () => undefined,
      started: () => undefined,
      startFailed: () => undefined,
    },
  });
  const ids = shortcutTestIds("transcribe");

  test("an empty recorder shows the empty state, dashed, without the unsaved label", () => {
    const markup = renderToStaticMarkup(
      <DraftRow
        actionId="transcribe"
        ids={ids}
        draft={draft("recording", null)}
        busy={false}
      />,
    );
    expect(
      markup.includes('data-testid="shortcut-settings-dictation-recording"'),
    ).toBe(true);
    expect(markup.includes('data-validity="empty"')).toBe(true);
    expect(markup.includes('data-draft="true"')).toBe(true);
    expect(markup.includes("border-dashed")).toBe(true);
    expect(
      markup.includes("shortcut-settings-dictation-recording-unsaved"),
    ).toBe(false);
  });

  test("a captured draft is dashed and labelled Not saved, unlike a stored shortcut", () => {
    const markup = renderToStaticMarkup(
      <DraftRow
        actionId="transcribe"
        ids={ids}
        draft={draft("invalid", "command_right")}
        busy={false}
      />,
    );
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-recording-unsaved"',
      ),
    ).toBe(true);
    expect(markup.includes("settings.general.shortcut.list.unsaved")).toBe(
      true,
    );
    expect(markup.includes("border-dashed")).toBe(true);
    const stored = renderToStaticMarkup(
      <ShortcutRow
        actionId="transcribe"
        shortcut="command_right"
        ids={ids}
        hasRemove
        canRemove
        busy={false}
        recordRequest={0}
        onRequestHandled={() => undefined}
        onEdit={noop}
      />,
    );
    expect(stored.includes("border-dashed")).toBe(false);
    expect(stored.includes("data-draft")).toBe(false);
  });

  test("an invalid recording is marked invalid", () => {
    const markup = renderToStaticMarkup(
      <DraftRow
        actionId="transcribe"
        ids={ids}
        draft={draft("invalid", "shift+alt+space")}
        busy={false}
      />,
    );
    expect(markup.includes('data-validity="invalid"')).toBe(true);
    expect(markup.includes('aria-invalid="true"')).toBe(true);
    expect(markup.includes("border-red-500")).toBe(true);
  });

  test("a recorder the system refused to start stays empty", () => {
    const markup = renderToStaticMarkup(
      <DraftRow
        actionId="transcribe"
        ids={ids}
        draft={draft("blocked", null)}
        busy={false}
      />,
    );
    expect(markup.includes('data-phase="blocked"')).toBe(true);
    expect(markup.includes('data-validity="empty"')).toBe(true);
    expect(markup.includes('data-state="idle"')).toBe(true);
  });
});

describe("controls under the list during a draft (native session N8)", () => {
  const ids = shortcutTestIds("transcribe");
  const captured = draftReducer(
    draftReducer(draftReducer(CLOSED_DRAFT, { type: "open" }), {
      type: "captured",
      shortcut: "command_right",
    }),
    { type: "checked", shortcut: "command_right", refusal: null },
  );
  const render = (state: typeof CLOSED_DRAFT) => {
    const controls = listControls(state, true, false);
    return renderToStaticMarkup(
      <AddRow
        ids={ids}
        controls={controls}
        isFull={false}
        onAdd={() => undefined}
        onSave={() => undefined}
        onDiscard={() => undefined}
        onReset={() => undefined}
        resetDisabled={!controls.isResetEnabled}
      />,
    );
  };

  test("without a draft: Add shortcut, Reset enabled, no Save or Discard", () => {
    const markup = render(CLOSED_DRAFT);
    expect(markup.includes("settings.general.shortcut.list.add")).toBe(true);
    expect(markup.includes("settings.general.shortcut.list.save")).toBe(false);
    expect(
      markup.includes("shortcut-settings-dictation-recording-discard"),
    ).toBe(false);
    expect(
      /data-testid="shortcut-settings-dictation-reset"[^>]*disabled/.test(
        markup,
      ),
    ).toBe(false);
  });

  test("a valid draft: Save and Discard replace Add, Reset is disabled", () => {
    const markup = render(captured);
    expect(markup.includes("settings.general.shortcut.list.add<")).toBe(false);
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-add" data-state="enabled"',
      ),
    ).toBe(true);
    expect(markup.includes("settings.general.shortcut.list.save")).toBe(true);
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-recording-discard"',
      ),
    ).toBe(true);
    expect(markup.includes("settings.general.shortcut.list.discardDraft")).toBe(
      true,
    );
    expect(
      /<button[^>]*disabled=""[^>]*data-testid="shortcut-settings-dictation-reset"|data-testid="shortcut-settings-dictation-reset"[^>]*disabled=""/.test(
        markup,
      ),
    ).toBe(true);
  });

  test("an empty draft: Save is unavailable until a valid combination", () => {
    const markup = render(draftReducer(CLOSED_DRAFT, { type: "open" }));
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-add" data-state="disabled"',
      ),
    ).toBe(true);
    expect(markup.includes("settings.general.shortcut.list.save")).toBe(true);
  });
});

describe("list messages", () => {
  test("the reason a recorder could not start is shown inline", () => {
    const markup = renderToStaticMarkup(
      <ListMessages
        ids={shortcutTestIds("transcribe")}
        conflictsFailed={false}
        error={describeStartFailure({ kind: "secure_input" })}
      />,
    );
    expect(markup.includes('role="alert"')).toBe(true);
    expect(markup.includes("secureInput.recorderBlocked")).toBe(true);
  });
});

describe("one-shot record requests (Change, Add)", () => {
  const ids = shortcutTestIds("transcribe");
  /** A mounted row chip: its own handled request, like `useListControl`'s ref. */
  const mountChip = () => ({ handled: { current: 0 }, starts: 0 });
  type Chip = ReturnType<typeof mountChip>;
  /** Runs the chip's request effect for the request the list gives it. */
  const runEffect = (
    chip: Chip,
    request: number,
    onHandled: (n: number) => void,
  ) =>
    actOnRecordRequest(request, chip.handled, () => (chip.starts += 1), {
      onRequestHandled: onHandled,
    });

  test("a row given a request renders idle until its chip handles it", () => {
    const markup = renderToStaticMarkup(
      <ShortcutRow
        actionId="transcribe"
        shortcut="alt+space"
        ids={ids}
        hasRemove
        canRemove
        busy={false}
        recordRequest={1}
        onRequestHandled={() => undefined}
        onEdit={noop}
      />,
    );
    expect(
      markup.includes('data-testid="shortcut-settings-dictation-shortcut"'),
    ).toBe(true);
    expect(markup.includes('data-state="idle"')).toBe(true);
  });

  test("Change starts the row once; the row remounted with the same shortcut does not record", () => {
    let list = requestRecording(NO_RECORD_REQUEST, "alt+space");
    const handle = (n: number) => (list = markRequestHandled(list, n));
    const first = mountChip();
    expect(runEffect(first, requestFor(list, "alt+space"), handle)).toBe(true);
    // StrictMode's second run and later re-renders never replay it.
    expect(runEffect(first, requestFor(list, "alt+space"), handle)).toBe(false);
    expect(first.starts).toBe(1);
    // The row leaves the list (CLI, another window) and comes back.
    list = dropUnlistedRequest(list, []);
    const remounted = mountChip();
    runEffect(remounted, requestFor(list, "alt+space"), handle);
    expect(remounted.starts).toBe(0);
  });

  test("a request pending while its row is gone is dropped before the row returns", () => {
    let list = requestRecording(NO_RECORD_REQUEST, "alt+space");
    list = dropUnlistedRequest(list, ["f5"]);
    const remounted = mountChip();
    runEffect(remounted, requestFor(list, "alt+space"), () => undefined);
    expect(remounted.starts).toBe(0);
  });

  test("the draft recorder remounted (keyboard implementation switch) does not record again", () => {
    let draft = requestRecording(NO_RECORD_REQUEST, "draft");
    const handle = (n: number) => (draft = markRequestHandled(draft, n));
    const tauriChip = mountChip();
    runEffect(tauriChip, requestFor(draft, "draft"), handle);
    expect(tauriChip.starts).toBe(1);
    const handyKeysChip = mountChip();
    runEffect(handyKeysChip, requestFor(draft, "draft"), handle);
    expect(handyKeysChip.starts).toBe(0);
  });
});
