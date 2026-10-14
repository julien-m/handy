/**
 * Rendered Post Process page and prompt shortcut list (005 with 004): the
 * page keeps the prompt groups and the "Dictation with AI processing"
 * shortcut row, and a prompt target's list keeps 0 to 2 shortcuts without
 * Reset. Run with `bun run test:prompt-settings`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type {
  ListedPrompt,
  PromptLibraryView,
  ShortcutBinding,
} from "@/bindings";

// Translations render as "key params" so the markup shows which text is used.
mock.module("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: unknown) =>
      typeof params === "object" ? `${key} ${JSON.stringify(params)}` : key,
  }),
  Trans: ({ i18nKey }: { i18nKey: string }) => i18nKey,
}));
mock.module("@tauri-apps/plugin-os", () => ({ type: () => "macos" }));
mock.module("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
mock.module("@/bindings", () => ({
  commands: { getShortcutConflicts: async () => [] },
}));
mock.module("sonner", () => ({ toast: { error: () => undefined } }));
mock.module("../../../stores/settingsStore", () => ({
  useSettingsStore: (select: (state: { revision: number }) => unknown) =>
    select({ revision: 1 }),
}));

const binding = (id: string, shortcuts: string[]): ShortcutBinding => ({
  id,
  name: id,
  description: `${id} description`,
  default_binding: shortcuts[0] ?? "",
  current_binding: shortcuts[0] ?? "",
  additional_bindings: shortcuts.slice(1),
});
const bindings: Record<string, ShortcutBinding> = {
  transcribe_with_post_process: binding("transcribe_with_post_process", [
    "alt+shift+space",
  ]),
  "prompt:one": binding("prompt:one", ["ctrl+alt+m"]),
  "prompt:two": binding("prompt:two", ["ctrl+alt+n", "ctrl+alt+b"]),
};
const prompt: ListedPrompt = {
  id: "one",
  name: "Notes",
  prompt: "Notes ${output}",
  provider_override: null,
  starter: false,
  issues: [],
  shortcuts: ["ctrl+alt+m"],
  readiness: {
    provider_id: "custom",
    model: "llama3.2",
    is_override: false,
    ready: true,
    missing: null,
  },
};
const library: PromptLibraryView = {
  default_prompt_id: "one",
  post_process_enabled: true,
  prompts: [prompt],
};
const settingValues: Record<string, unknown> = {
  bindings,
  keyboard_implementation: "tauri",
  post_process_enabled: true,
  post_process_prompts: [{ id: "one", name: "Notes" }],
  post_process_providers: [],
};
mock.module("../../../hooks/useSettings", () => ({
  useSettings: () => ({
    settings: { post_process_providers: [] },
    getSetting: (key: string) => settingValues[key],
    editBindingShortcuts: async () => null,
    refusedShortcuts: [],
    isUpdating: () => false,
    isLoading: false,
    postProcessModelOptions: {},
  }),
}));
mock.module("./use-prompt-library", () => ({
  usePromptLibrary: () => ({
    library,
    loadedRevision: 1,
    loadState: "ready",
    reload: () => undefined,
  }),
}));

const { PostProcessingSettings } = await import("./PostProcessingSettings");
const { ShortcutListEditor } = await import("../shortcut-list-editor");

const has = (markup: string, testId: string): boolean =>
  markup.includes(`data-testid="${testId}"`);

describe("Post Process page", () => {
  const markup = renderToStaticMarkup(<PostProcessingSettings />);

  test("shows the prompt groups and the library", () => {
    expect(has(markup, "prompt-settings-screen")).toBe(true);
    expect(has(markup, "prompt-settings-prompt-name")).toBe(true);
    expect(markup).toContain("Notes");
  });

  test("keeps the AI dictation shortcut row of 004", () => {
    expect(has(markup, "shortcut-settings-ai-dictation")).toBe(true);
    expect(has(markup, "shortcut-settings-ai-dictation-shortcut")).toBe(true);
    expect(has(markup, "shortcut-settings-ai-dictation-add")).toBe(true);
    expect(has(markup, "shortcut-settings-ai-dictation-reset")).toBe(true);
  });
});

const editorIds = {
  list: "prompt-settings-editor-form-shortcuts",
  row: () => "prompt-settings-editor-form-shortcut",
  add: "prompt-settings-editor-form-add-shortcut",
};
const renderPromptList = (actionId: string) =>
  renderToStaticMarkup(
    <ShortcutListEditor actionId={actionId} layout="row" testIds={editorIds} />,
  );

describe("prompt shortcut list (prompt:<id>)", () => {
  test("one shortcut: removable, room for another, no Reset", () => {
    const markup = renderPromptList("prompt:one");
    expect(has(markup, editorIds.list)).toBe(true);
    expect(has(markup, "prompt-settings-editor-form-shortcut")).toBe(true);
    expect(markup).toMatch(
      /data-testid="prompt-settings-editor-form-shortcut-remove" aria-label="settings.general.shortcut.list.remove" aria-disabled="false"/,
    );
    expect(markup).toContain("settings.general.shortcut.list.add");
    expect(markup).not.toContain("settings.general.shortcut.list.reset");
  });

  test("two shortcuts: the limit of 2 disables adding", () => {
    const markup = renderPromptList("prompt:two");
    expect(markup).toMatch(
      /data-testid="prompt-settings-editor-form-add-shortcut" data-state="idle" aria-label="settings.general.shortcut.list.addLimit \{&quot;limit&quot;:2\}" aria-disabled="true"/,
    );
  });

  test("no shortcut yet: only the add recorder", () => {
    const markup = renderPromptList("prompt:new");
    expect(has(markup, "prompt-settings-editor-form-shortcut")).toBe(false);
    expect(has(markup, editorIds.add)).toBe(true);
  });
});
