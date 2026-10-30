/**
 * Retry of a failed vocabulary commit (feature 003): the focus goes to the term input after a
 * successful Retry of an Add or a removal, and a Retry of a failed Apply sends only an applicable
 * merge that adds terms, reports why otherwise, and confirms a success like Apply.
 * Run with `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import type { ImportPreview, VocabularyImportPreview } from "@/bindings";
import type { VocabularyStore } from "@/stores/vocabulary-store";
import { retryApply, retryThenFocus } from "./vocabulary-retry";

const preview = (
  added: string[],
  overrides: Partial<ImportPreview> = {},
): VocabularyImportPreview => ({
  revision: 3,
  applicable: (overrides.issues ?? []).length === 0,
  preview: {
    mode: "merge",
    added: added.map((term, index) => ({ index, term })),
    duplicates: [],
    issues: [],
    candidate: added,
    over_capacity: false,
    ...overrides,
  },
});

/** A store whose Retry answers `succeeds` and counts its calls. */
const fakeStore = (
  current: VocabularyImportPreview | null,
  succeeds: boolean,
) => {
  const calls = { retry: 0 };
  const state = {
    pasteText: "OpenClaw",
    previewText: "OpenClaw",
    preview: current,
    retry: async () => {
      calls.retry += 1;
      return succeeds;
    },
  };
  // Only getState is read by the retry actions; the rest of the store API is not used.
  const store = { getState: () => state } as unknown as VocabularyStore;
  return { store, calls };
};

/** Run a Retry of a failed Apply and record what it reported. */
const runApplyRetry = async (store: VocabularyStore) => {
  const reports: string[] = [];
  await retryApply(store, {
    blocked: (block) => void reports.push(`${block.tone}:${block.note.key}`),
    applied: () => void reports.push("applied"),
  });
  return reports;
};

// @spec AC-001: Paste box with preview, no partial write — .specs/features/003-recognition-vocabulary/spec.md#ac-001
describe("Retry of a failed Apply", () => {
  test("an applicable merge is sent and confirmed like Apply", async () => {
    const { store, calls } = fakeStore(preview(["OpenClaw"]), true);
    assert.deepEqual(await runApplyRetry(store), ["applied"]);
    assert.equal(calls.retry, 1);
  });

  test("a merge that adds nothing is not sent", async () => {
    const { store, calls } = fakeStore(preview([]), true);
    assert.deepEqual(await runApplyRetry(store), [
      "info:settings.vocabulary.pasteBox.note.nothingToAdd",
    ]);
    assert.equal(calls.retry, 0);
  });

  test("a preview that cannot be applied is not sent and says why", async () => {
    const issues = [{ index: 1, issue: "forbidden_character" as const }];
    const { store, calls } = fakeStore(preview(["OpenClaw"], { issues }), true);
    assert.deepEqual(await runApplyRetry(store), [
      "warning:settings.vocabulary.pasteBox.note.fixOne",
    ]);
    assert.equal(calls.retry, 0);
  });

  test("a Retry that fails again reports nothing more (the error stays)", async () => {
    const { store, calls } = fakeStore(preview(["OpenClaw"]), false);
    assert.deepEqual(await runApplyRetry(store), []);
    assert.equal(calls.retry, 1);
  });
});

// @spec FR-001: Vocabulary edits with atomic commit — .specs/features/003-recognition-vocabulary/spec.md#fr-001
describe("Retry of a failed Add or removal", () => {
  test("the term input takes the focus after a successful Retry only", async () => {
    let focused = 0;
    const focus = () => void (focused += 1);
    await retryThenFocus(fakeStore(null, true).store, focus)();
    assert.equal(focused, 1);
    await retryThenFocus(fakeStore(null, false).store, focus)();
    assert.equal(focused, 1);
  });
});
