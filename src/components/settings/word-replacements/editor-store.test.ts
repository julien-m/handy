/**
 * Replacement editor stores against stubbed commands (feature 002): the list draft and its
 * single Save at the starting revision, revision conflicts, refreshes after a change made
 * elsewhere, the text preview and Apply, file import and export, and the unsaved-edit guard.
 * Run with `bun run test:word-replacements` (Bun's test runner, for its module mocking).
 */
import assert from "node:assert/strict";
import { beforeEach, describe, test } from "bun:test";
import { conflict, resetStubs, rule, stub, view } from "./editor-test-stubs";

// Dynamic imports: the stores must load after the stubs module registered its mock.
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
  resetStubs();
  stub.pickReply = { status: "ok", data: null };
  stub.exportReply = { status: "ok", data: null };
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
    stub.checkIssues = [
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
    stub.current = view(5, [
      ...stub.current.rules,
      rule("n", "Open Claw", "OpenClaw"),
    ]);
    assert.equal(await editor().save(), true);
    assert.equal(stub.saveCalls.length, 1);
    assert.equal(stub.saveCalls[0][1], 4);
    assert.deepEqual(stub.saveCalls[0][0][1], {
      id: null,
      recognized: "Open Claw",
      replacement: "OpenClaw",
    });
    assert.equal(editor().commit, "saved");
    assert.equal(isListDirty(editor()), false);
  });

  test("an invalid draft disables Save", async () => {
    stub.checkIssues = [
      { rule_index: 0, field: "replacement", kind: "too_short" },
    ];
    editor().editRule("a", "replacement", "");
    await settle();
    assert.equal(canSave(editor()), false);
    assert.equal(await editor().save(), false);
    assert.equal(stub.saveCalls.length, 0);
  });

  test("a revision conflict keeps the draft and its base revision", async () => {
    editor().editRule("a", "replacement", "Claude Code CLI");
    await settle();
    // The rules really changed elsewhere, so the conflict stands.
    stub.current = view(6, [rule("z", "Cloud Opus", "Claude Opus")]);
    stub.saveReplies.push({ status: "error", error: conflict(6) });
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
    stub.current = view(7, [rule("z", "Cloud Opus", "Claude Opus")]);
    await editor().followSettingsChange(7);
    assert.equal(editor().revision, 7);
    assert.equal(editor().draft[0].recognized, "Cloud Opus");
  });

  test("a dirty editor keeps its draft, so Save reports the conflict", async () => {
    editor().editRule("a", "replacement", "Mine");
    await settle();
    stub.current = view(7, []);
    await editor().followSettingsChange(7);
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
    assert.deepEqual(stub.applyCalls[0], [
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
    // The same text now previews differently: the rules changed elsewhere.
    stub.previewExtra = { ignored_lines: 1 };
    stub.applyReplies.push({ status: "error", error: conflict(9) });
    assert.equal(await useReplacementText.getState().apply(), false);
    assert.equal(useReplacementText.getState().failure?.conflict, true);
    assert.equal(useReplacementText.getState().text, "other");
  });
});

describe("files", () => {
  test("a file that is not UTF-8 is refused with its name", async () => {
    stub.pickReply = {
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
    stub.pickReply = {
      status: "ok",
      data: { file_name: "rules-team.csv", text: "a;b\n" },
    };
    await useReplacementFiles.getState().pickImport();
    assert.deepEqual(last(stub.previewCalls), ["a;b\n", "merge"]);
    await useReplacementFiles.getState().setMode("replace");
    assert.deepEqual(last(stub.previewCalls), ["a;b\n", "replace"]);
    assert.equal(canApplyImport(useReplacementFiles.getState()), true);
    assert.equal(await useReplacementFiles.getState().applyImport(), true);
    assert.deepEqual(last(stub.applyCalls), ["a;b\n", "replace", 4]);
    assert.equal(useReplacementFiles.getState().file, null);
  });

  test("an import conflict keeps the dialog open", async () => {
    stub.pickReply = {
      status: "ok",
      data: { file_name: "r.txt", text: "a;b\n" },
    };
    await useReplacementFiles.getState().pickImport();
    stub.previewExtra = { ignored_lines: 1 };
    stub.applyReplies.push({ status: "error", error: conflict(8) });
    assert.equal(await useReplacementFiles.getState().applyImport(), false);
    assert.equal(useReplacementFiles.getState().failure?.conflict, true);
    assert.notEqual(useReplacementFiles.getState().file, null);
  });

  test("export reports the written file or the typed refusal", async () => {
    stub.exportReply = {
      status: "ok",
      data: { file_name: "r.txt", count: 1, durable: true },
    };
    assert.deepEqual(await useReplacementFiles.getState().exportRules(), {
      status: "exported",
      fileName: "r.txt",
      count: 1,
      durable: true,
    });
    stub.exportReply = {
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
    stub.exportReply = { status: "ok", data: null };
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
