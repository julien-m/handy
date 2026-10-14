/**
 * Vocabulary editor notices and focus rules: a failed commit states "unchanged" only without a
 * durable outcome and offers no Retry after a committed one, an unread list is never the empty
 * state, every secondary button shows a visible keyboard focus, and focus after a removal lands
 * on the right term. Run with `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { Occurrence, SettingsCommandError } from "@/bindings";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
const { CommitFailure, ListLoading, ListReadFailure } = await import(
  "./vocabulary-notices"
);
const { VocabularyApplyBar } = await import("./vocabulary-paste");
const { VocabularyStatusView } = await import("./vocabulary-status");
const { failureNotice, FOCUS_RING, removalFocusPosition } = await import(
  "./vocabulary-model"
);

const SCREEN = "vocabulary-settings";
const SAVED = "settings.vocabulary.errors.saved";
const RETRY = ">settings.vocabulary.retry<";

const refusal = (
  overrides: Partial<SettingsCommandError>,
): SettingsCommandError => ({
  code: "persistence_failed",
  message: "Refused.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: null,
  ...overrides,
});

const failure = (error: SettingsCommandError | null) =>
  renderToStaticMarkup(
    <CommitFailure
      screen={SCREEN}
      error={error}
      errorId="err"
      onRetry={() => undefined}
    />,
  );

describe("commit failure outcomes", () => {
  test("no outcome: the saved vocabulary is unchanged and Retry is offered", () => {
    assert.deepEqual(failureNotice(refusal({})), {
      unchanged: true,
      retry: true,
    });
    const markup = failure(refusal({}));
    assert.equal(markup.includes(SAVED), true);
    assert.equal(markup.includes(RETRY), true);
    assert.equal(failure(null).includes(SAVED), true);
  });

  test("unknown outcome: no 'unchanged' claim, Retry after the reread", () => {
    const error = refusal({ code: "unavailable", outcome: "unknown" });
    assert.deepEqual(failureNotice(error), { unchanged: false, retry: true });
    const markup = failure(error);
    assert.equal(markup.includes(SAVED), false);
    assert.equal(markup.includes("settings.vocabulary.errors.unknown"), true);
    assert.equal(markup.includes(RETRY), true);
  });

  test("committed outcome: the write is done, no 'unchanged' and no Retry", () => {
    const error = refusal({ code: "internal_error", outcome: "committed" });
    assert.deepEqual(failureNotice(error), { unchanged: false, retry: false });
    const markup = failure(error);
    assert.equal(markup.includes(SAVED), false);
    assert.equal(markup.includes("settings.vocabulary.errors.committed"), true);
    assert.equal(markup.includes(RETRY), false);
  });
});

describe("saved list read", () => {
  test("a failed read shows the error with Retry, never the empty state", () => {
    const markup = renderToStaticMarkup(
      <ListReadFailure screen={SCREEN} onRetry={() => undefined} />,
    );
    assert.equal(markup.includes(`data-testid="${SCREEN}-list-error"`), true);
    assert.equal(markup.includes("settings.vocabulary.listFailed"), true);
    assert.equal(markup.includes(RETRY), true);
    assert.equal(markup.includes("settings.vocabulary.empty.title"), false);
  });

  test("a pending read shows a loading line", () => {
    const markup = renderToStaticMarkup(<ListLoading screen={SCREEN} />);
    assert.equal(markup.includes(`data-testid="${SCREEN}-loading"`), true);
    assert.equal(markup.includes("settings.vocabulary.loading"), true);
  });
});

describe("visible keyboard focus", () => {
  test("Cancel and every Retry carry the focus ring", () => {
    const bar = renderToStaticMarkup(
      <VocabularyApplyBar
        text=""
        preview={null}
        fresh={false}
        applying={false}
        onCancel={() => undefined}
        onApply={() => undefined}
      />,
    );
    const cancel = bar.match(/<button[^>]*-cancel"[^>]*>/)?.[0] ?? "";
    assert.equal(cancel.includes(FOCUS_RING), true);
    const status = renderToStaticMarkup(
      <VocabularyStatusView
        status={{ kind: "failed", error: null }}
        screen={SCREEN}
        model="Small"
        onRetry={() => undefined}
      />,
    );
    const retries = [status, failure(refusal({}))].map(
      (markup) => markup.match(/<button[^>]*>/)?.[0] ?? "",
    );
    for (const retry of retries) assert.equal(retry.includes(FOCUS_RING), true);
  });
});

describe("focus after a removal", () => {
  const terms = (...names: string[]): Occurrence[] =>
    names.map((term, index) => ({ index, term, issue: null }));

  test("a removed term hands focus to the next one, or to the input", () => {
    const after = terms("Claude Code", "Penflow");
    const target = { index: 0, term: "OpenClaw", removed: true };
    assert.equal(removalFocusPosition(after, target), 0);
    assert.equal(removalFocusPosition(after, { ...target, index: 2 }), null);
  });

  test("a failed removal keeps focus on the same term after the reread", () => {
    // The reread list gained a term in front: the term moved to position 1.
    const after = terms("Penflow", "OpenClaw", "Claude Code");
    const target = { index: 0, term: "OpenClaw", removed: false };
    assert.equal(removalFocusPosition(after, target), 1);
    const unchanged = terms("OpenClaw", "Claude Code");
    assert.equal(removalFocusPosition(unchanged, target), 0);
  });
});
