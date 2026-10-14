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
const { DraftRow } = await import("./shortcut-list-rows");
const { shortcutTestIds } = await import("./shortcut-list-feedback");

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
    expect(markup.includes('data-testid="shortcut-settings-dictation"')).toBe(
      true,
    );
    expect(
      markup.includes('data-testid="shortcut-settings-dictation-title"'),
    ).toBe(true);
    expect(
      count(markup, 'data-testid="shortcut-settings-dictation-shortcut"'),
    ).toBe(4);
    expect(
      count(
        markup,
        'data-testid="shortcut-settings-dictation-shortcut-remove"',
      ),
    ).toBe(4);
  });

  test("Add is disabled and the limit hint explains why", () => {
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-add" data-state="disabled"',
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
        'data-testid="shortcut-settings-dictation-list" data-state="idle"',
      ),
    ).toBe(true);
    expect(markup.includes("shortcut-settings-dictation-recording")).toBe(
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
        ids={shortcutTestIds("cancel")}
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
        ids={shortcutTestIds("transcribe")}
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
  const draft = (phase: "recording" | "invalid", shortcut: string | null) => ({
    state: { phase, shortcut, refusal: null },
    recordRequest: 0,
    open: () => undefined,
    capture: noop,
    stop: () => undefined,
    commit: noop,
    discard: () => undefined,
  });
  const ids = shortcutTestIds("transcribe");

  test("an empty recorder shows the empty state without discard", () => {
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
    expect(
      markup.includes("shortcut-settings-dictation-recording-discard"),
    ).toBe(false);
  });

  test("an invalid recording is marked invalid and can be discarded", () => {
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
    expect(
      markup.includes(
        'data-testid="shortcut-settings-dictation-recording-discard"',
      ),
    ).toBe(true);
  });
});
