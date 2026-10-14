/**
 * Vocabulary editor store against stubbed commands: one commit at a time with the shown revision,
 * typed refusals kept with the saved list, rereads after conflicts, and stale list, status and
 * preview answers discarded. Run with `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import type {
  Result,
  SettingsCommandError,
  VocabularyImportPreview,
  VocabularyList,
  VocabularyStatusReport,
} from "@/bindings";
import {
  createVocabularyStore,
  needsReload,
  type VocabularyApi,
} from "./vocabulary-store";

type Deferred<T> = { promise: Promise<T>; resolve: (value: T) => void };
const deferred = <T>(): Deferred<T> => {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};

const list = (revision: number, terms: string[]): VocabularyList => ({
  revision,
  terms: terms.map((term, index) => ({ index, term, issue: null })),
});

const refusal = (
  overrides: Partial<SettingsCommandError>,
): SettingsCommandError => ({
  code: "validation_failed",
  message: "Refused.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: null,
  ...overrides,
});

const report = (
  overrides: Partial<VocabularyStatusReport>,
): VocabularyStatusReport => ({
  model_id: "small",
  mode: "batch",
  status: "used_during_recognition",
  reason: null,
  mechanism: "whisper_initial_prompt",
  budget: { value: 223, unit: "tokens" },
  terms_sent: ["Claude Code"],
  terms_not_sent: [],
  evidence_kind: "prepared",
  ...overrides,
});

const preview = (
  revision: number,
  added: string[],
  issues: VocabularyImportPreview["preview"]["issues"] = [],
): VocabularyImportPreview => ({
  revision,
  applicable: issues.length === 0,
  preview: {
    mode: "merge",
    added: added.map((term, index) => ({ index, term })),
    duplicates: [],
    issues,
    candidate: added,
    over_capacity: false,
  },
});

type Commit = Result<VocabularyList, SettingsCommandError>;
const ok = (data: VocabularyList): Commit => ({ status: "ok", data });
const refused = (error: SettingsCommandError): Commit => ({
  status: "error",
  error,
});

/** Stub API: each command answers from its queue, in call order, and records its calls. */
const stubApi = () => {
  const calls: Array<[string, unknown[]]> = [];
  const queues = {
    lists: [] as Array<Promise<VocabularyList>>,
    statuses: [] as Array<
      Promise<Result<VocabularyStatusReport, SettingsCommandError>>
    >,
    commits: [] as Array<Promise<Commit>>,
    previews: [] as Array<Promise<VocabularyImportPreview>>,
  };
  const record =
    <T>(name: string, queue: Array<Promise<T>>, fallback: () => T) =>
    (...args: unknown[]): Promise<T> => {
      calls.push([name, args]);
      return queue.shift() ?? Promise.resolve(fallback());
    };
  const okStatus = () => ({ status: "ok" as const, data: report({}) });
  const api: VocabularyApi = {
    getVocabulary: record("getVocabulary", queues.lists, () => list(1, [])),
    getVocabularyStatus: record("status", queues.statuses, okStatus),
    addVocabularyTerm: record("add", queues.commits, () => ok(list(2, []))),
    removeVocabularyOccurrence: record("remove", queues.commits, () =>
      ok(list(2, [])),
    ),
    previewVocabularyImport: record("preview", queues.previews, () =>
      preview(1, []),
    ),
    applyVocabularyImport: record("apply", queues.commits, () =>
      ok(list(2, [])),
    ),
  };
  const named = (name: string) => calls.filter(([call]) => call === name);
  return { api, queues, named };
};

const loadedStore = async (terms: string[], revision = 1) => {
  const stub = stubApi();
  stub.queues.lists.push(Promise.resolve(list(revision, terms)));
  const store = createVocabularyStore(stub.api);
  await store.getState().loadList();
  return { store, ...stub };
};

describe("list reads", () => {
  test("an older revision never replaces a newer list", async () => {
    const { store, queues } = await loadedStore(["OpenClaw"], 5);
    queues.lists.push(Promise.resolve(list(4, [])));
    await store.getState().loadList();
    assert.equal(store.getState().list?.revision, 5);
  });

  test("only the latest read updates the list", async () => {
    const { store, queues } = await loadedStore([], 1);
    const slow = deferred<VocabularyList>();
    queues.lists.push(slow.promise, Promise.resolve(list(3, ["Penflow"])));
    const first = store.getState().loadList();
    await store.getState().loadList();
    slow.resolve(list(2, ["stale"]));
    await first;
    assert.deepEqual(
      store.getState().list?.terms.map((t) => t.term),
      ["Penflow"],
    );
  });
});

describe("commits", () => {
  test("Add commits the draft at the shown revision and refreshes the status", async () => {
    const { store, queues, named } = await loadedStore([], 7);
    queues.commits.push(Promise.resolve(ok(list(8, ["Claude Code"]))));
    store.getState().setDraft("Claude Code");
    assert.equal(await store.getState().add(), true);
    assert.deepEqual(named("add")[0][1], ["Claude Code", 7]);
    const state = store.getState();
    assert.equal(state.list?.revision, 8);
    assert.equal(state.phase, "success");
    assert.equal(state.draft, "");
    assert.equal(named("status").length, 1);
  });

  test("a pending commit is busy and cannot dispatch another", async () => {
    const { store, queues, named } = await loadedStore(["OpenClaw"]);
    const slow = deferred<Commit>();
    queues.commits.push(slow.promise);
    const first = store.getState().remove(0, "OpenClaw");
    assert.equal(store.getState().phase, "loading");
    assert.equal(await store.getState().remove(0, "OpenClaw"), false);
    slow.resolve(ok(list(2, [])));
    assert.equal(await first, true);
    assert.equal(named("remove").length, 1);
  });

  test("a duplicate is refused, the list kept, and typing clears the error", async () => {
    const { store, queues } = await loadedStore(["Claude Code"]);
    const error = refusal({
      items: [{ index: 0, code: "term_duplicate", message: "Duplicate." }],
    });
    queues.commits.push(Promise.resolve(refused(error)));
    store.getState().setDraft("claude code");
    assert.equal(await store.getState().add(), false);
    assert.equal(store.getState().phase, "error");
    assert.deepEqual(store.getState().failure?.error, error);
    assert.equal(store.getState().list?.terms.length, 1);
    store.getState().setDraft("OpenClaw");
    assert.equal(store.getState().failure, null);
    assert.equal(store.getState().phase, "idle");
  });

  test("a revision conflict rereads the list and Retry uses the new revision", async () => {
    const { store, queues, named } = await loadedStore(["OpenClaw"], 3);
    queues.commits.push(
      Promise.resolve(
        refused(refusal({ code: "revision_conflict", current_revision: 4 })),
      ),
    );
    queues.lists.push(Promise.resolve(list(4, ["OpenClaw", "Penflow"])));
    assert.equal(await store.getState().remove(0, "OpenClaw"), false);
    assert.equal(store.getState().list?.revision, 4);
    queues.commits.push(Promise.resolve(ok(list(5, ["Penflow"]))));
    assert.equal(await store.getState().retry(), true);
    assert.deepEqual(
      named("remove").map(([, args]) => args),
      [
        [0, "OpenClaw", 3],
        [0, "OpenClaw", 4],
      ],
    );
  });

  test("a thrown command is an error that rereads the saved list", async () => {
    const { store, queues, named } = await loadedStore(["OpenClaw"]);
    queues.commits.push(Promise.reject(new Error("ipc down")));
    queues.lists.push(Promise.resolve(list(1, ["OpenClaw"])));
    assert.equal(await store.getState().remove(0, "OpenClaw"), false);
    assert.equal(store.getState().phase, "error");
    assert.equal(named("getVocabulary").length, 2);
    assert.equal(store.getState().list?.terms.length, 1);
  });
});

describe("durable outcomes", () => {
  for (const [code, outcome] of [
    ["unavailable", "unknown"],
    ["internal_error", "committed"],
  ] as const) {
    test(`an ${outcome} outcome rereads the saved list`, async () => {
      const { store, queues, named } = await loadedStore(["OpenClaw"]);
      queues.commits.push(Promise.resolve(refused(refusal({ code, outcome }))));
      queues.lists.push(Promise.resolve(list(2, [])));
      assert.equal(await store.getState().remove(0, "OpenClaw"), false);
      assert.equal(named("getVocabulary").length, 2);
      assert.equal(store.getState().list?.revision, 2);
      assert.equal(store.getState().failure?.error?.outcome, outcome);
    });
  }
});

describe("status", () => {
  test("a superseded status answer is discarded", async () => {
    const { store, queues } = await loadedStore([]);
    const slow =
      deferred<Result<VocabularyStatusReport, SettingsCommandError>>();
    queues.statuses.push(
      slow.promise,
      Promise.resolve({
        status: "ok",
        data: report({ status: "not_supported" }),
      }),
    );
    const first = store.getState().loadStatus();
    await store.getState().loadStatus();
    slow.resolve({ status: "ok", data: report({}) });
    await first;
    const status = store.getState().status;
    assert.equal(
      status.kind === "ready" && status.report.status,
      "not_supported",
    );
  });

  test("an unverified budget is a failed read, not one of the five statuses", async () => {
    const { store, queues } = await loadedStore([]);
    const error = refusal({
      code: "internal_error",
      detail_reason: "context_budget_unverified",
    });
    queues.statuses.push(Promise.resolve({ status: "error", error }));
    await store.getState().loadStatus();
    assert.deepEqual(store.getState().status, { kind: "failed", error });
  });
});

describe("paste", () => {
  test("an answer to older text is discarded and Apply waits for a fresh preview", async () => {
    const { store, queues, named } = await loadedStore([]);
    store.getState().openPaste();
    store.getState().setPasteText("Claude Code");
    const slow = deferred<VocabularyImportPreview>();
    queues.previews.push(slow.promise);
    const pending = store.getState().loadPreview();
    store.getState().setPasteText("Claude Code\nOpenClaw");
    slow.resolve(preview(1, ["Claude Code"]));
    await pending;
    assert.equal(store.getState().preview, null);
    assert.equal(await store.getState().apply(), false);
    assert.equal(named("apply").length, 0);
  });

  test("an invalid line keeps Apply from committing", async () => {
    const { store, queues, named } = await loadedStore([]);
    store.getState().openPaste();
    store.getState().setPasteText("OpenClaw\n<prompt> template");
    queues.previews.push(
      Promise.resolve(
        preview(1, ["OpenClaw"], [{ index: 1, issue: "forbidden_character" }]),
      ),
    );
    await store.getState().loadPreview();
    assert.equal(await store.getState().apply(), false);
    assert.equal(named("apply").length, 0);
  });

  test("Apply commits the previewed text in merge mode and closes the paste box", async () => {
    const { store, queues, named } = await loadedStore(["Claude Code"], 2);
    store.getState().openPaste();
    store.getState().setPasteText("Claude Sonnet\nOpenClaw");
    queues.previews.push(
      Promise.resolve(preview(2, ["Claude Sonnet", "OpenClaw"])),
    );
    await store.getState().loadPreview();
    queues.commits.push(
      Promise.resolve(
        ok(list(3, ["Claude Code", "Claude Sonnet", "OpenClaw"])),
      ),
    );
    assert.equal(await store.getState().apply(), true);
    assert.deepEqual(named("apply")[0][1], [
      "Claude Sonnet\nOpenClaw",
      "merge",
      2,
    ]);
    const state = store.getState();
    assert.equal(state.pasteOpen, false);
    assert.equal(state.pasteText, "");
    assert.equal(state.list?.terms.length, 3);
  });

  test("Apply commits at its preview's revision, not a newer shown list", async () => {
    const { store, queues, named } = await loadedStore([], 2);
    store.getState().openPaste();
    store.getState().setPasteText("OpenClaw");
    queues.previews.push(Promise.resolve(preview(2, ["OpenClaw"])));
    await store.getState().loadPreview();
    // Another window or the CLI changed the list before the preview was reread.
    queues.lists.push(Promise.resolve(list(3, ["Penflow"])));
    await store.getState().loadList();
    queues.commits.push(
      Promise.resolve(refused(refusal({ code: "revision_conflict" }))),
    );
    assert.equal(await store.getState().apply(), false);
    assert.equal(named("apply")[0][1][2], 2);
  });

  test("a conflicting Apply rereads the list and the preview", async () => {
    const { store, queues, named } = await loadedStore([], 2);
    store.getState().openPaste();
    store.getState().setPasteText("OpenClaw");
    queues.previews.push(Promise.resolve(preview(2, ["OpenClaw"])));
    await store.getState().loadPreview();
    queues.commits.push(
      Promise.resolve(refused(refusal({ code: "revision_conflict" }))),
    );
    queues.lists.push(Promise.resolve(list(3, ["OpenClaw"])));
    queues.previews.push(Promise.resolve(preview(3, [])));
    assert.equal(await store.getState().apply(), false);
    assert.equal(store.getState().list?.revision, 3);
    assert.equal(named("preview").length, 2);
    assert.deepEqual(store.getState().preview?.preview.added, []);
    // The reread merge adds nothing: Retry sends no empty merge.
    assert.equal(await store.getState().retry(), false);
    assert.equal(named("apply").length, 1);
  });

  test("Retry of an Apply sends nothing while the reread preview is not applicable", async () => {
    const { store, queues, named } = await loadedStore([], 2);
    store.getState().openPaste();
    store.getState().setPasteText("OpenClaw\n<x>");
    queues.previews.push(Promise.resolve(preview(2, ["OpenClaw", "<x>"])));
    await store.getState().loadPreview();
    queues.commits.push(
      Promise.resolve(refused(refusal({ outcome: "unknown" }))),
    );
    const issue = { index: 1, issue: "forbidden_character" as const };
    queues.previews.push(Promise.resolve(preview(3, ["OpenClaw"], [issue])));
    assert.equal(await store.getState().apply(), false);
    assert.equal(await store.getState().retry(), false);
    assert.equal(named("apply").length, 1);
  });
});

test("rereads follow conflicts, missing rows and unconfirmed outcomes only", () => {
  assert.equal(needsReload(refusal({ code: "revision_conflict" })), true);
  assert.equal(needsReload(refusal({ code: "not_found" })), true);
  assert.equal(needsReload(refusal({ outcome: "unknown" })), true);
  assert.equal(needsReload(refusal({ outcome: "committed" })), true);
  assert.equal(needsReload(null), true);
  assert.equal(needsReload(refusal({ code: "limit_exceeded" })), false);
  assert.equal(needsReload(refusal({})), false);
});
