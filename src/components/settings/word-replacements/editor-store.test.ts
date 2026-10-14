/**
 * Replacement editor stores against stubbed commands (feature 002): the list draft and its
 * single Save at the starting revision, revision conflicts, refreshes after a change made
 * elsewhere, the text preview and Apply, file import and export, and the unsaved-edit guard.
 * Run with `bun run test:word-replacements` (Bun's test runner, for its module mocking).
 */
import assert from "node:assert/strict";
import { beforeEach, describe, mock, test } from "bun:test";
import type {
  ImportMode,
  ReplacementPreview,
  ReplacementRule,
  ReplacementRulesView,
  RuleDraft,
  RuleIssue,
  SettingsCommandError,
} from "@/bindings";

type Reply<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: SettingsCommandError };

const rule = (id: string, recognized: string, replacement: string) => ({
  id,
  recognized,
  replacement,
});
const view = (
  revision: number,
  rules: ReplacementRule[],
): ReplacementRulesView => ({
  revision,
  rules,
});
const emptyPreview = (
  overrides: Partial<ReplacementPreview>,
): ReplacementPreview => ({
  mode: "replace",
  input_sha256: "0",
  lines: [],
  set_issues: [],
  candidate: [],
  is_noop: false,
  ignored_lines: 0,
  error_lines: 0,
  omitted_lines: 0,
  ...overrides,
});
const conflict = (current: number): SettingsCommandError => ({
  code: "revision_conflict",
  message: "Settings changed since the expected revision.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: current,
});

// Each scenario sets the stubbed replies; calls are recorded in order.
let current = view(4, [rule("a", "Cloud Code", "Claude Code")]);
let checkIssues: RuleIssue[] = [];
let saveReply: Reply<ReplacementRulesView> | null = null;
let applyReply: Reply<ReplacementRulesView> | null = null;
let previewRevision = 4;
let pickReply: unknown = { status: "ok", data: null };
let exportReply: unknown = { status: "ok", data: null };
const saveCalls: Array<[RuleDraft[], number]> = [];
const applyCalls: Array<[string, ImportMode, number]> = [];
const previewCalls: Array<[string, ImportMode]> = [];

const commands = {
  getWordReplacements: async () => current,
  checkWordReplacements: async () => checkIssues,
  saveWordReplacements: async (rules: RuleDraft[], revision: number) => {
    saveCalls.push([rules, revision]);
    return (
      saveReply ?? { status: "ok", data: view(revision + 1, current.rules) }
    );
  },
  previewWordReplacements: async (input: string, mode: ImportMode) => {
    previewCalls.push([input, mode]);
    const preview = emptyPreview({ mode, is_noop: input.endsWith("same") });
    return { status: "ok", data: { revision: previewRevision, preview } };
  },
  applyWordReplacements: async (
    input: string,
    mode: ImportMode,
    revision: number,
  ) => {
    applyCalls.push([input, mode, revision]);
    return applyReply ?? { status: "ok", data: view(revision + 1, []) };
  },
  importWordReplacementsFile: async () => pickReply,
  exportWordReplacementsFile: async () => exportReply,
};
mock.module("@/bindings", () => ({ commands }));
// Dynamic imports: the stores must load after the mock above is registered.
const { canSave, isListDirty, useReplacementEditor } = await import(
  "./editor-store"
);
const { canApplyText, useReplacementText } = await import("./text-store");
const { canApplyImport, useReplacementFiles } = await import("./file-store");
const guards = await import("@/stores/unsaved-changes");

const editor = () => useReplacementEditor.getState();
const last = <T>(calls: T[]): T | undefined => calls[calls.length - 1];
const settle = () => new Promise((done) => setTimeout(done, 0));

beforeEach(async () => {
  current = view(4, [rule("a", "Cloud Code", "Claude Code")]);
  checkIssues = [];
  saveReply = null;
  applyReply = null;
  previewRevision = 4;
  saveCalls.length = 0;
  applyCalls.length = 0;
  previewCalls.length = 0;
  useReplacementEditor.setState({
    revision: null,
    commit: "idle",
    view: "list",
  });
  await editor().loadRules();
  useReplacementFiles.getState().closeImport();
});

describe("list draft", () => {
  test("loading shows the saved rules as a clean draft", () => {
    assert.equal(editor().load, "ready");
    assert.equal(editor().revision, 4);
    assert.equal(isListDirty(editor()), false);
    assert.equal(canSave(editor()), false);
  });

  test("a rule the shared validator refuses is not added", async () => {
    checkIssues = [
      { rule_index: 1, field: "recognized", kind: "duplicate_key" },
    ];
    const added = await editor().addRule({
      recognized: "cloud  code",
      replacement: "X",
    });
    assert.equal(added, false);
    assert.equal(editor().draft.length, 1);
  });

  test("add then Save commits the whole draft once at the starting revision", async () => {
    assert.equal(
      await editor().addRule({
        recognized: "Open Claw",
        replacement: "OpenClaw",
      }),
      true,
    );
    await settle();
    assert.equal(canSave(editor()), true);
    current = view(5, [...current.rules, rule("n", "Open Claw", "OpenClaw")]);
    assert.equal(await editor().save(), true);
    assert.equal(saveCalls.length, 1);
    assert.equal(saveCalls[0][1], 4);
    assert.deepEqual(saveCalls[0][0][1], {
      id: null,
      recognized: "Open Claw",
      replacement: "OpenClaw",
    });
    assert.equal(editor().commit, "saved");
    assert.equal(isListDirty(editor()), false);
  });

  test("an invalid draft disables Save", async () => {
    checkIssues = [{ rule_index: 0, field: "replacement", kind: "too_short" }];
    editor().editRule("a", "replacement", "");
    await settle();
    assert.equal(canSave(editor()), false);
    assert.equal(await editor().save(), false);
    assert.equal(saveCalls.length, 0);
  });

  test("a revision conflict keeps the draft and its base revision", async () => {
    editor().editRule("a", "replacement", "Claude Code CLI");
    await settle();
    saveReply = { status: "error", error: conflict(6) };
    assert.equal(await editor().save(), false);
    assert.equal(editor().failure?.conflict, true);
    assert.equal(editor().revision, 4);
    assert.equal(editor().draft[0].replacement, "Claude Code CLI");
  });

  test("discard restores the saved rules", async () => {
    editor().removeRule("a");
    await settle();
    editor().discard();
    assert.equal(isListDirty(editor()), false);
  });
});

describe("changes made elsewhere", () => {
  test("a clean editor shows the newer rules", async () => {
    current = view(7, [rule("z", "Cloud Opus", "Claude Opus")]);
    await editor().refreshAfterChange(7);
    assert.equal(editor().revision, 7);
    assert.equal(editor().draft[0].recognized, "Cloud Opus");
  });

  test("a dirty editor keeps its draft, so Save reports the conflict", async () => {
    editor().editRule("a", "replacement", "Mine");
    await settle();
    current = view(7, []);
    await editor().refreshAfterChange(7);
    assert.equal(editor().revision, 4);
    assert.equal(editor().draft[0].replacement, "Mine");
  });
});

describe("text view", () => {
  test("Apply commits the previewed text at the preview's revision", async () => {
    const text = useReplacementText.getState();
    text.open(editor().saved);
    text.setText("recognized;replacement\nOpen Claw;OpenClaw");
    assert.equal(canApplyText(useReplacementText.getState()), false);
    await useReplacementText.getState().requestPreview();
    assert.equal(canApplyText(useReplacementText.getState()), true);
    assert.equal(await useReplacementText.getState().apply(), true);
    assert.deepEqual(applyCalls[0], [
      "recognized;replacement\nOpen Claw;OpenClaw",
      "replace",
      4,
    ]);
    assert.equal(editor().revision, 5);
  });

  test("an edit after the preview disables Apply until previewed again", async () => {
    useReplacementText.getState().open(editor().saved);
    useReplacementText.getState().setText("recognized;replacement\nA;B");
    await useReplacementText.getState().requestPreview();
    useReplacementText.getState().setText("recognized;replacement\nA;C");
    assert.equal(canApplyText(useReplacementText.getState()), false);
  });

  test("a no-op preview has nothing to apply; a conflict keeps the text", async () => {
    useReplacementText.getState().setText("same");
    await useReplacementText.getState().requestPreview();
    assert.equal(canApplyText(useReplacementText.getState()), false);
    useReplacementText.getState().setText("other");
    await useReplacementText.getState().requestPreview();
    applyReply = { status: "error", error: conflict(9) };
    assert.equal(await useReplacementText.getState().apply(), false);
    assert.equal(useReplacementText.getState().failure?.conflict, true);
    assert.equal(useReplacementText.getState().text, "other");
  });
});

describe("files", () => {
  test("a file that is not UTF-8 is refused with its name", async () => {
    pickReply = {
      status: "error",
      error: {
        file_name: "notes.rtf",
        error: {
          ...conflict(0),
          code: "validation_failed",
          reason: "invalid_encoding",
        },
      },
    };
    await useReplacementFiles.getState().pickImport();
    assert.deepEqual(useReplacementFiles.getState().refusal, {
      fileName: "notes.rtf",
      kind: "encoding",
    });
    assert.equal(useReplacementFiles.getState().phase, "idle");
  });

  test("merge preview, replace re-preview, then Apply at the preview revision", async () => {
    pickReply = {
      status: "ok",
      data: { file_name: "rules-team.csv", text: "a;b\n" },
    };
    await useReplacementFiles.getState().pickImport();
    assert.deepEqual(last(previewCalls), ["a;b\n", "merge"]);
    await useReplacementFiles.getState().setMode("replace");
    assert.deepEqual(last(previewCalls), ["a;b\n", "replace"]);
    assert.equal(canApplyImport(useReplacementFiles.getState()), true);
    assert.equal(await useReplacementFiles.getState().applyImport(), true);
    assert.deepEqual(last(applyCalls), ["a;b\n", "replace", 4]);
    assert.equal(useReplacementFiles.getState().file, null);
  });

  test("an import conflict keeps the dialog open", async () => {
    pickReply = { status: "ok", data: { file_name: "r.txt", text: "a;b\n" } };
    await useReplacementFiles.getState().pickImport();
    applyReply = { status: "error", error: conflict(8) };
    assert.equal(await useReplacementFiles.getState().applyImport(), false);
    assert.equal(useReplacementFiles.getState().failure?.conflict, true);
    assert.notEqual(useReplacementFiles.getState().file, null);
  });

  test("export reports the written file or the typed refusal", async () => {
    exportReply = {
      status: "ok",
      data: { file_name: "r.txt", count: 1, durable: true },
    };
    assert.deepEqual(await useReplacementFiles.getState().exportRules(), {
      status: "exported",
      fileName: "r.txt",
      count: 1,
      durable: true,
    });
    exportReply = {
      status: "error",
      error: {
        file_name: "link.txt",
        error: {
          ...conflict(0),
          code: "persistence_failed",
          detail_reason: "export_symbolic_link",
        },
      },
    };
    const failed = await useReplacementFiles.getState().exportRules();
    assert.equal(failed.status, "failed");
    exportReply = { status: "ok", data: null };
    assert.equal(
      (await useReplacementFiles.getState().exportRules()).status,
      "cancelled",
    );
  });
});

describe("navigation guard", () => {
  test("a registered dirty editor blocks until discarded", () => {
    let dirty = true;
    const unregister = guards.registerUnsavedChangesGuard({
      isDirty: () => dirty,
      discard: () => {
        dirty = false;
      },
    });
    assert.equal(guards.hasUnsavedChanges(), true);
    guards.discardUnsavedChanges();
    assert.equal(guards.hasUnsavedChanges(), false);
    unregister();
    dirty = true;
    assert.equal(guards.hasUnsavedChanges(), false);
  });
});
