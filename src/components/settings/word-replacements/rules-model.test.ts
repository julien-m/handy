/**
 * Replacement editor rules (feature 002): draft edits, dirty state, the bulk text, issue
 * translation keys and contract screen ids.
 * Run with `bun run test:word-replacements`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import type { ReplacementRule } from "@/bindings";
import {
  EXCHANGE_HEADER,
  draftOf,
  exchangeText,
  isDraftDirty,
  issueKey,
  itemKey,
  rowIssues,
  ruleDrafts,
  screenId,
  setIssues,
  withAddedRule,
  withEditedRule,
  withoutRule,
} from "./rules-model";

const saved: ReplacementRule[] = [
  { id: "a", recognized: "Cloud Code", replacement: "Claude Code" },
  { id: "b", recognized: "Cloud Sonnet", replacement: "Claude Sonnet" },
  { id: "c", recognized: "Open Claw", replacement: "OpenClaw" },
];

describe("draft", () => {
  test("the saved rules make a clean draft keyed by id", () => {
    const draft = draftOf(saved);
    assert.deepEqual(
      draft.map((rule) => rule.key),
      ["a", "b", "c"],
    );
    assert.equal(isDraftDirty(saved, draft), false);
  });

  test("add appends a new rule without id and makes the draft dirty", () => {
    const draft = withAddedRule(
      draftOf(saved),
      { recognized: "Cloud Opus", replacement: "Claude Opus" },
      "new-1",
    );
    assert.equal(draft.length, 4);
    assert.deepEqual(ruleDrafts(draft)[3], {
      id: null,
      recognized: "Cloud Opus",
      replacement: "Claude Opus",
    });
    assert.equal(isDraftDirty(saved, draft), true);
  });

  test("edit keeps the id and the position", () => {
    const draft = withEditedRule(draftOf(saved), "b", "replacement", "Sonnet");
    assert.deepEqual(ruleDrafts(draft)[1], {
      id: "b",
      recognized: "Cloud Sonnet",
      replacement: "Sonnet",
    });
    assert.equal(isDraftDirty(saved, draft), true);
    const reverted = withEditedRule(draft, "b", "replacement", "Claude Sonnet");
    assert.equal(isDraftDirty(saved, reverted), false);
  });

  test("delete removes one row; a reorder is also a change", () => {
    const draft = withoutRule(draftOf(saved), "a");
    assert.deepEqual(
      draft.map((rule) => rule.id),
      ["b", "c"],
    );
    assert.equal(isDraftDirty(saved, draft), true);
    const reordered = [...draftOf(saved)].reverse();
    assert.equal(isDraftDirty(saved, reordered), true);
  });
});

describe("bulk text", () => {
  test("the header then one recognized;replacement line per rule", () => {
    assert.equal(
      exchangeText(saved),
      `${EXCHANGE_HEADER}\nCloud Code;Claude Code\nCloud Sonnet;Claude Sonnet\nOpen Claw;OpenClaw`,
    );
    assert.equal(exchangeText([]), EXCHANGE_HEADER);
  });

  test("a replacement may contain ; and stays after the first separator", () => {
    const text = exchangeText([
      { id: "x", recognized: "csv", replacement: "a;b" },
    ]);
    assert.equal(text.split("\n")[1], "csv;a;b");
  });
});

describe("issue keys", () => {
  test("field issues map to their own messages", () => {
    assert.equal(
      issueKey("recognized", "duplicate_key"),
      "settings.replacements.issues.duplicate",
    );
    assert.equal(
      issueKey("recognized", "too_short"),
      "settings.replacements.issues.recognizedEmpty",
    );
    assert.equal(
      issueKey("replacement", "too_long"),
      "settings.replacements.issues.replacementTooLong",
    );
    assert.equal(
      issueKey("line", "missing_separator"),
      "settings.replacements.issues.missingSeparator",
    );
  });

  test("whole-set issues win whatever the field; identity issues are generic", () => {
    assert.equal(
      issueKey("rules", "too_many_rules"),
      "settings.replacements.issues.tooManyRules",
    );
    assert.equal(
      issueKey("recognized", "unicode_data_unavailable"),
      "settings.replacements.issues.unicodeUnavailable",
    );
    assert.equal(
      issueKey("id", "duplicate_id"),
      "settings.replacements.issues.invalidRule",
    );
  });

  test("refused Save positions map like the validator's issues", () => {
    assert.equal(
      itemKey({ code: "duplicate_recognized" }),
      "settings.replacements.issues.duplicate",
    );
    assert.equal(
      itemKey({ code: "shortcut_session_busy" }),
      "settings.replacements.issues.invalidRule",
    );
  });

  test("row and set issues are separated by their index", () => {
    const issues = [
      {
        rule_index: 1,
        field: "recognized" as const,
        kind: "duplicate_key" as const,
      },
      {
        rule_index: null,
        field: "rules" as const,
        kind: "too_many_rules" as const,
      },
    ];
    assert.equal(rowIssues(issues, 1).length, 1);
    assert.equal(rowIssues(issues, 0).length, 0);
    assert.equal(setIssues(issues)[0].kind, "too_many_rules");
  });
});

describe("contract screens", () => {
  test("the text view, the list or the empty state", () => {
    assert.equal(screenId("text", false), "replacement-settings-bulk");
    assert.equal(screenId("list", true), "replacement-settings");
    assert.equal(screenId("list", false), "replacement-settings-empty");
  });
});
