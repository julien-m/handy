/**
 * Replacement editor stores and the global settings revision (feature 002): a change of another
 * setting never turns Save, Apply or an import into a false conflict, a real change of the rules
 * still does and keeps Save disabled until Reload, an uncertain Save is settled by a reread,
 * edits, Discard and navigation are locked while a commit is in flight, and a late validator
 * answer never marks rules it was not asked about.
 * Run with `bun run test:word-replacements` (Bun's test runner, for its module mocking).
 */
import assert from "node:assert/strict";
import { beforeEach, describe, test } from "bun:test";
import type { SettingsCommandError } from "@/bindings";
import { conflict, resetStubs, rule, stub, view } from "./editor-test-stubs";

// Dynamic imports: the stores must load after the stubs module registered its mock.
const { canSave, isListDirty, useReplacementEditor } = await import(
  "./editor-store"
);
const { useReplacementText } = await import("./text-store");
const { useReplacementFiles } = await import("./file-store");
const { discardEditor, isCommitInFlight } = await import(
  "./use-editor-lifecycle"
);
const guards = await import("@/stores/unsaved-changes");

const editor = () => useReplacementEditor.getState();
const settle = () => new Promise((done) => setTimeout(done, 0));
const saved = () => [rule("a", "Cloud Code", "Claude Code")];
const unknownOutcome: SettingsCommandError = {
  ...conflict(0),
  code: "unavailable",
  outcome: "unknown",
};

/** Edit the only rule's replacement and wait for the validator. */
const editReplacement = async (replacement: string): Promise<void> => {
  editor().editRule("a", "replacement", replacement);
  await settle();
};

beforeEach(async () => {
  resetStubs();
  useReplacementEditor.setState({
    revision: null,
    commit: "idle",
    failure: null,
    view: "list",
  });
  await editor().loadRules();
  useReplacementText.getState().open(editor().saved);
  useReplacementFiles.getState().closeImport();
});

describe("another setting changed", () => {
  test("a clean editor adopts the revision; the next Save uses it", async () => {
    stub.current = view(5, saved());
    await editor().followSettingsChange(5);
    assert.equal(editor().revision, 5);
    await editReplacement("Claude Code CLI");
    assert.equal(await editor().save(), true);
    assert.equal(stub.saveCalls[0][1], 5);
  });

  test("a dirty draft also adopts it and keeps its edits", async () => {
    await editReplacement("Mine");
    stub.current = view(5, saved());
    await editor().followSettingsChange(5);
    assert.equal(editor().revision, 5);
    assert.equal(editor().draft[0].replacement, "Mine");
    assert.equal(await editor().save(), true);
    assert.equal(stub.saveCalls[0][1], 5);
  });

  test("a change during the Save is retried once at the current revision", async () => {
    await editReplacement("Mine");
    stub.current = view(5, saved());
    stub.saveReplies.push({ status: "error", error: conflict(5) });
    assert.equal(await editor().save(), true);
    assert.deepEqual(
      stub.saveCalls.map(([, revision]) => revision),
      [4, 5],
    );
  });
});

describe("the rules changed elsewhere", () => {
  test("a dirty draft keeps its base revision and Save reports the conflict", async () => {
    await editReplacement("Mine");
    stub.current = view(5, [rule("z", "Cloud Opus", "Claude Opus")]);
    await editor().followSettingsChange(5);
    assert.equal(editor().revision, 4);
    stub.saveReplies.push({ status: "error", error: conflict(5) });
    assert.equal(await editor().save(), false);
    assert.equal(stub.saveCalls.length, 1);
    assert.equal(editor().failure?.conflict, true);
  });

  test("after a conflict, an edit keeps Save disabled until Reload", async () => {
    await editReplacement("Mine");
    stub.current = view(5, [rule("z", "Cloud Opus", "Claude Opus")]);
    stub.saveReplies.push({ status: "error", error: conflict(5) });
    await editor().save();
    await editor().addRule({
      recognized: "Open Claw",
      replacement: "OpenClaw",
    });
    await settle();
    assert.equal(editor().failure?.conflict, true);
    assert.equal(canSave(editor()), false);
    await editor().loadRules();
    assert.equal(editor().failure, null);
    assert.equal(editor().revision, 5);
  });
});

test("after a conflict, Discard shows the rules saved now", async () => {
  await editReplacement("Mine");
  stub.current = view(5, [rule("z", "Cloud Opus", "Claude Opus")]);
  stub.saveReplies.push({ status: "error", error: conflict(5) });
  await editor().save();
  editor().discard();
  await settle();
  assert.equal(editor().revision, 5);
  assert.equal(editor().draft[0].recognized, "Cloud Opus");
  assert.equal(editor().failure, null);
});

describe("uncertain Save outcome", () => {
  test("rules equal to the sent draft are accepted as saved", async () => {
    await editReplacement("Mine");
    stub.saveReplies.push({ status: "error", error: unknownOutcome });
    stub.current = view(5, [rule("a", "Cloud Code", "Mine")]);
    assert.equal(await editor().save(), false);
    await settle();
    assert.equal(editor().commit, "saved");
    assert.equal(editor().revision, 5);
    assert.equal(isListDirty(editor()), false);
  });

  test("unchanged rules: the draft keeps its base revision, saved rules unchanged", async () => {
    await editReplacement("Mine");
    stub.saveReplies.push({ status: "error", error: unknownOutcome });
    stub.current = view(5, saved());
    await editor().save();
    await settle();
    assert.equal(editor().revision, 4);
    assert.equal(editor().failure?.uncertain, false);
    assert.equal(editor().failure?.conflict, false);
    assert.equal(editor().draft[0].replacement, "Mine");
  });

  test("other rules: a conflict, never the reread revision", async () => {
    await editReplacement("Mine");
    stub.saveReplies.push({ status: "error", error: unknownOutcome });
    stub.current = view(5, [rule("z", "Cloud Opus", "Claude Opus")]);
    await editor().save();
    await settle();
    assert.equal(editor().revision, 4);
    assert.equal(editor().failure?.conflict, true);
    assert.equal(canSave(editor()), false);
  });
});

describe("edits while a commit is in flight", () => {
  test("list edits are ignored during a Save, then the saved rules show", async () => {
    await editReplacement("Mine");
    let release = () => {};
    stub.saveGate = new Promise((done) => (release = done));
    const saving = editor().save();
    editor().editRule("a", "replacement", "Typed during Save");
    editor().removeRule("a");
    assert.equal(editor().draft[0].replacement, "Mine");
    release();
    assert.equal(await saving, true);
    assert.equal(stub.saveCalls.length, 1);
  });

  test("the text is read-only while Apply runs", async () => {
    const text = useReplacementText.getState();
    text.setText("recognized;replacement\nA;B");
    await useReplacementText.getState().requestPreview();
    useReplacementText.setState({ applying: true });
    useReplacementText.getState().setText("typed during Apply");
    assert.equal(
      useReplacementText.getState().text,
      "recognized;replacement\nA;B",
    );
    useReplacementText.setState({ applying: false });
  });
});

describe("Discard and navigation while a commit is in flight", () => {
  test("Discard during a Save keeps the lock; the Save settles", async () => {
    await editReplacement("Mine");
    let release = () => {};
    stub.saveGate = new Promise((done) => (release = done));
    const saving = editor().save();
    editor().discard();
    discardEditor();
    assert.equal(editor().commit, "saving");
    assert.equal(editor().draft[0].replacement, "Mine");
    assert.equal(isCommitInFlight(), true);
    release();
    assert.equal(await saving, true);
    assert.equal(editor().commit, "saved");
  });

  test("Discard and Cancel during an Apply keep the text read-only", async () => {
    const typed = "recognized;replacement\nA;B";
    editor().setView("text");
    useReplacementText.getState().setText(typed);
    await useReplacementText.getState().requestPreview();
    let release = () => {};
    stub.applyGate = new Promise((done) => (release = done));
    const applying = useReplacementText.getState().apply();
    discardEditor();
    useReplacementText.getState().cancel();
    assert.equal(useReplacementText.getState().applying, true);
    assert.equal(useReplacementText.getState().text, typed);
    assert.equal(editor().view, "text");
    release();
    assert.equal(await applying, true);
    assert.equal(useReplacementText.getState().applying, false);
  });

  test("section navigation is not offered while the editor commits", async () => {
    await editReplacement("Mine");
    let busy = true;
    let discarded = 0;
    const unregister = guards.registerUnsavedChangesGuard({
      isDirty: () => true,
      isBusy: () => busy,
      discard: () => void (discarded += 1),
    });
    assert.equal(guards.hasCommitInFlight(), true);
    guards.discardUnsavedChanges();
    assert.equal(discarded, 0);
    busy = false;
    assert.equal(guards.hasCommitInFlight(), false);
    guards.discardUnsavedChanges();
    assert.equal(discarded, 1);
    unregister();
  });
});

describe("late validator answers", () => {
  test("a check of a replaced draft does not mark the reloaded rules", async () => {
    let release = () => {};
    stub.checkGate = new Promise((done) => (release = done));
    stub.checkIssues = [
      { rule_index: 0, field: "replacement", kind: "duplicate_key" },
    ];
    editor().editRule("a", "replacement", "Mine");
    stub.checkIssues = [];
    await editor().loadRules();
    release();
    await settle();
    assert.equal(editor().issues.length, 0);
    assert.equal(editor().checking, false);
    assert.equal(isListDirty(editor()), false);
  });
});

describe("Apply and import after another setting changed", () => {
  test("an unchanged preview applies once at the fresh revision", async () => {
    useReplacementText.getState().setText("recognized;replacement\nA;B");
    await useReplacementText.getState().requestPreview();
    stub.previewRevision = 5;
    stub.applyReplies.push({ status: "error", error: conflict(5) });
    assert.equal(await useReplacementText.getState().apply(), true);
    assert.deepEqual(
      stub.applyCalls.map(([, , revision]) => revision),
      [4, 5],
    );
  });

  test("a changed preview stays a conflict, with no second Apply", async () => {
    useReplacementText.getState().setText("recognized;replacement\nA;B");
    await useReplacementText.getState().requestPreview();
    stub.previewExtra = { ignored_lines: 1 };
    stub.applyReplies.push({ status: "error", error: conflict(5) });
    assert.equal(await useReplacementText.getState().apply(), false);
    assert.equal(stub.applyCalls.length, 1);
    assert.equal(useReplacementText.getState().failure?.conflict, true);
  });

  test("an import previewing the same is applied at the fresh revision", async () => {
    stub.pickReply = {
      status: "ok",
      data: { file_name: "r.txt", text: "a;b\n" },
    };
    await useReplacementFiles.getState().pickImport();
    stub.previewRevision = 6;
    stub.applyReplies.push({ status: "error", error: conflict(6) });
    assert.equal(await useReplacementFiles.getState().applyImport(), true);
    assert.deepEqual(stub.applyCalls[1], ["a;b\n", "merge", 6]);
  });
});
