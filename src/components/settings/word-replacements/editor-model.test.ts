/**
 * Pure decisions of the replacement editor (feature 002): following committed settings changes
 * at the global revision, settling an uncertain Save, the conflict kept after an edit, the screen
 * kept after the last deletion and the focus after a deletion (keyboard AC "Keyboard delete").
 * Run with `bun run test:word-replacements`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import type { ReplacementRule } from "@/bindings";
import {
  failureAfterEdit,
  focusAfterDelete,
  followDecision,
  rulesMatchDraft,
  sameRules,
  settledFailure,
  showsRuleList,
  uncertainOutcome,
} from "./editor-model";
import type { CommitFailure } from "./preview-model";

const saved: ReplacementRule[] = [
  { id: "a", recognized: "Cloud Code", replacement: "Claude Code" },
  { id: "b", recognized: "Cloud Sonnet", replacement: "Claude Sonnet" },
];
const changed: ReplacementRule[] = [
  { id: "a", recognized: "Cloud Code", replacement: "Claude Code CLI" },
];
const follow = {
  ready: true,
  saving: false,
  dirty: false,
  revision: 4,
  saved,
};
const failure: CommitFailure = {
  key: "settings.errors.generic",
  conflict: false,
  uncertain: true,
  items: [],
};

describe("rule comparison", () => {
  test("same rules need the same ids, content and order", () => {
    assert.equal(sameRules(saved, [...saved]), true);
    assert.equal(sameRules(saved, [...saved].reverse()), false);
    assert.equal(sameRules(saved, changed), false);
    assert.equal(sameRules(saved, [{ ...saved[0], id: "x" }, saved[1]]), false);
  });

  test("a sent draft matches its saved rules; new rules take any id", () => {
    const sent = [
      { id: "a", recognized: "Cloud Code", replacement: "Claude Code" },
      { id: null, recognized: "Open Claw", replacement: "OpenClaw" },
    ];
    const stored = [
      saved[0],
      { id: "n1", recognized: "Open Claw", replacement: "OpenClaw" },
    ];
    assert.equal(rulesMatchDraft(sent, stored), true);
    assert.equal(rulesMatchDraft(sent, saved), false);
    assert.equal(rulesMatchDraft(sent.slice(0, 1), stored), false);
  });
});

describe("following a committed settings change", () => {
  test("another setting changed: a clean or dirty draft moves to the new revision", () => {
    const next = { revision: 5, rules: [...saved] };
    assert.equal(followDecision(follow, next), "rebase");
    assert.equal(followDecision({ ...follow, dirty: true }, next), "rebase");
  });

  test("the rules changed: a clean list shows them, a dirty draft keeps its base", () => {
    const next = { revision: 5, rules: changed };
    assert.equal(followDecision(follow, next), "replace");
    assert.equal(followDecision({ ...follow, dirty: true }, next), "ignore");
  });

  test("an older view, a Save in flight or an unloaded editor changes nothing", () => {
    const same = { revision: 4, rules: changed };
    assert.equal(followDecision(follow, same), "ignore");
    const next = { revision: 5, rules: changed };
    assert.equal(followDecision({ ...follow, saving: true }, next), "ignore");
    assert.equal(followDecision({ ...follow, ready: false }, next), "ignore");
  });
});

describe("uncertain Save outcome", () => {
  const sent = [
    { id: "a", recognized: "Cloud Code", replacement: "Claude Code CLI" },
  ];

  test("rules equal to the sent draft: committed", () => {
    assert.equal(uncertainOutcome(sent, saved, changed), "committed");
  });

  test("rules still the saved ones: not committed, saved rules unchanged", () => {
    assert.equal(uncertainOutcome(sent, saved, saved), "notCommitted");
    assert.deepEqual(settledFailure(failure, "notCommitted"), {
      ...failure,
      uncertain: false,
    });
  });

  test("other rules: a conflict that keeps Save disabled until Reload", () => {
    const other = [{ id: "z", recognized: "Cloud Opus", replacement: "X" }];
    assert.equal(uncertainOutcome(sent, saved, other), "conflict");
    const settled = settledFailure(failure, "conflict");
    assert.equal(settled.conflict, true);
    assert.equal(settled.uncertain, true);
  });
});

describe("edits after a refusal", () => {
  test("a conflict stays; any other refusal clears", () => {
    const conflict = { ...failure, conflict: true };
    assert.equal(failureAfterEdit(conflict), conflict);
    assert.equal(failureAfterEdit(failure), null);
    assert.equal(failureAfterEdit(null), null);
  });
});

describe("screen and focus after a deletion", () => {
  test("the list stays while it has rules, an open add form or a changed draft", () => {
    assert.equal(showsRuleList(2, false, false), true);
    assert.equal(showsRuleList(0, true, false), true);
    // Deleting the last saved rule: no row left, but the draft changed.
    assert.equal(showsRuleList(0, false, true), true);
    assert.equal(showsRuleList(0, false, false), false);
  });

  test("focus goes to the next rule, else to the add form", () => {
    const keys = ["a", "b", "c"];
    assert.deepEqual(focusAfterDelete(keys, "a"), { kind: "rule", key: "b" });
    // The last of several rules: the add form, never the previous rule.
    assert.deepEqual(focusAfterDelete(keys, "c"), { kind: "addForm" });
    assert.deepEqual(focusAfterDelete(["a"], "a"), { kind: "addForm" });
    assert.deepEqual(focusAfterDelete(keys, "missing"), { kind: "addForm" });
  });
});
