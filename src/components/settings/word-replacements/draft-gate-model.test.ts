/**
 * Guard dialog of the replacement editor (feature 002, AC-001): a guarded transition waits for
 * Save, Discard or Cancel; nothing is offered, discarded or cancelled while a Save or an Apply is
 * in flight; and a transition never runs after a Cancel or a refused commit.
 * Run with `bun run test:word-replacements`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import { createDraftGate, type PendingTransition } from "./draft-gate-model";

/** A guard over a fake editor whose commit is held until `finish`. */
const fakeGate = (dirty = true) => {
  const state = {
    dirty,
    busy: false,
    discards: 0,
    runs: 0,
    shown: null as PendingTransition | null,
  };
  let finish: (committed: boolean) => void = () => undefined;
  const gate = createDraftGate({
    isDirty: () => state.dirty,
    isBusy: () => state.busy,
    discard: () => void (state.discards += 1),
    commit: () => {
      state.busy = true;
      return new Promise<boolean>((resolve) => {
        finish = (committed) => {
          state.busy = false;
          resolve(committed);
        };
      });
    },
    onChange: (pending) => void (state.shown = pending),
  });
  const run = () => void (state.runs += 1);
  return {
    state,
    gate,
    run,
    finish: (committed: boolean) => finish(committed),
  };
};

// @spec AC-001: Unsaved edits are saved or discarded before a view change — .specs/features/002-word-replacements/spec.md#ac-001
describe("guard dialog", () => {
  test("clean editor: the transition runs at once, no dialog", () => {
    const { state, gate, run } = fakeGate(false);
    gate.request(run);
    assert.equal(state.runs, 1);
    assert.equal(state.shown, null);
  });

  test("Save then the waiting transition, once", async () => {
    const { state, gate, run, finish } = fakeGate();
    gate.request(run);
    assert.notEqual(state.shown, null);
    const saving = gate.save();
    finish(true);
    await saving;
    assert.equal(state.runs, 1);
    assert.equal(state.shown, null);
  });

  test("a refused Save closes the dialog without the transition", async () => {
    const { state, gate, run, finish } = fakeGate();
    gate.request(run);
    const saving = gate.save();
    finish(false);
    await saving;
    assert.equal(state.runs, 0);
    assert.equal(state.shown, null);
  });

  test("while the Save runs, Cancel, Discard and new transitions do nothing", async () => {
    const { state, gate, run, finish } = fakeGate();
    gate.request(run);
    const saving = gate.save();
    gate.cancel();
    gate.discard();
    gate.request(run);
    assert.notEqual(state.shown, null);
    assert.equal(state.discards, 0);
    assert.equal(state.runs, 0);
    finish(true);
    await saving;
    assert.equal(state.runs, 1);
  });

  test("a commit running elsewhere: no dialog and no transition", () => {
    const { state, gate, run } = fakeGate();
    state.busy = true;
    gate.request(run);
    assert.equal(state.shown, null);
    assert.equal(state.runs, 0);
  });

  test("a Cancel before the commit answers: the transition never runs", async () => {
    const { state, gate, run, finish } = fakeGate();
    gate.request(run);
    const saving = gate.save();
    // A commit that did not mark itself busy (refused before sending) leaves Cancel active.
    state.busy = false;
    gate.cancel();
    finish(true);
    await saving;
    assert.equal(state.runs, 0);
    assert.equal(state.shown, null);
  });

  test("Discard drops the edits, then runs the transition", () => {
    const { state, gate, run } = fakeGate();
    gate.request(run);
    gate.discard();
    assert.equal(state.discards, 1);
    assert.equal(state.runs, 1);
    assert.equal(state.shown, null);
  });
});
