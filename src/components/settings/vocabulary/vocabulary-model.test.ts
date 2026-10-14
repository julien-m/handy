/**
 * Presentation rules of the vocabulary editor: mockup screens and test IDs, Behavioral AC visual
 * states, the five statuses, the paste preview and refusal wording. Run with
 * `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type {
  ImportPreview,
  SettingsCommandError,
  VocabularyImportPreview,
  VocabularyStatusReport,
} from "@/bindings";
import {
  canAddTerm,
  canRetry,
  chipFlagKey,
  commitErrorKey,
  isTermRefusal,
  listScreen,
  overBudgetIndices,
  termInputState,
  VOCABULARY_SCREENS,
} from "./vocabulary-model";
import {
  applyLabelKey,
  applyNote,
  applyRetryBlock,
  applyState,
  previewRows,
  previewSummary,
} from "./vocabulary-paste-model";
import {
  notSentGroups,
  sentTotal,
  statusText,
  statusTone,
} from "./vocabulary-status-model";

const report = (
  overrides: Partial<VocabularyStatusReport>,
): VocabularyStatusReport => ({
  model_id: "small",
  mode: "batch",
  status: "used_during_recognition",
  reason: null,
  mechanism: "whisper_initial_prompt",
  budget: { value: 223, unit: "tokens" },
  terms_sent: [],
  terms_not_sent: [],
  evidence_kind: "prepared",
  ...overrides,
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

// The spec's paste scenario plus the mockup's invalid line.
const specPaste: ImportPreview = {
  mode: "merge",
  added: [
    { index: 0, term: "Claude Code" },
    { index: 1, term: "Claude Sonnet" },
    { index: 3, term: "OpenClaw" },
  ],
  duplicates: [{ index: 2, term: "claude code" }],
  issues: [{ index: 4, issue: "forbidden_character" }],
  candidate: ["Claude Code", "Claude Sonnet", "OpenClaw"],
  over_capacity: false,
};
const wrap = (preview: ImportPreview): VocabularyImportPreview => ({
  revision: 1,
  preview,
  applicable: preview.issues.length === 0 && !preview.over_capacity,
});

const here = path.dirname(fileURLToPath(import.meta.url));

describe("screens and test IDs", () => {
  test("the list view matches the empty, unsupported or default mockup", () => {
    assert.equal(listScreen(0, "not_supported"), "vocabulary-settings-empty");
    assert.equal(
      listScreen(3, "not_supported"),
      "vocabulary-settings-unsupported",
    );
    assert.equal(
      listScreen(3, "used_during_recognition"),
      "vocabulary-settings",
    );
    assert.equal(listScreen(3, null), "vocabulary-settings");
    // An unread list (loading or failed) is never shown as the empty screen.
    assert.equal(listScreen(null, null), "vocabulary-settings");
  });

  test("every contract test ID of the vocabulary screens is emitted", () => {
    const root = path.resolve(here, "../../../..");
    const contract = JSON.parse(
      fs.readFileSync(
        path.join(root, "penflow/flow-ui-contract/contract.json"),
        "utf8",
      ),
    ) as { nodes: Array<{ screen_id: string; test_id: string }> };
    const sources = [
      "../CustomWords.tsx",
      "vocabulary-terms.tsx",
      "vocabulary-status.tsx",
      "vocabulary-paste.tsx",
    ]
      .map((file) => fs.readFileSync(path.join(here, file), "utf8"))
      .join("\n");
    const screens = Object.values(VOCABULARY_SCREENS) as string[];
    const ids = contract.nodes.filter((node) =>
      screens.includes(node.screen_id),
    );
    assert.ok(ids.length > 50);
    for (const { screen_id: screen, test_id: testId } of ids) {
      const suffix = testId.slice(screen.length + 1);
      // Test IDs are written as `${screen}-<suffix>` (or `${SCREEN}-` on the paste screen).
      assert.equal(sources.includes(`}-${suffix}\``), true);
    }
  });
});

describe("Behavioral AC visual states", () => {
  test("term input: empty, invalid while refused, valid otherwise", () => {
    assert.equal(termInputState("  ", false), "empty");
    assert.equal(termInputState("claude code", true), "invalid");
    assert.equal(termInputState("OpenClaw", false), "valid");
  });

  test("Add is unavailable for blank input or while a commit is pending", () => {
    assert.equal(canAddTerm("", false), false);
    assert.equal(canAddTerm(" \t", false), false);
    assert.equal(canAddTerm("OpenClaw", true), false);
    assert.equal(canAddTerm("OpenClaw", false), true);
  });

  test("Apply: enabled, disabled for an invalid or stale preview, loading", () => {
    const valid = wrap({ ...specPaste, issues: [] });
    assert.equal(applyState(valid, true, false), "enabled");
    assert.equal(applyState(wrap(specPaste), true, false), "disabled");
    assert.equal(applyState(valid, false, false), "disabled");
    assert.equal(applyState(null, true, false), "disabled");
    assert.equal(applyState(valid, true, true), "loading");
    const nothing = wrap({ ...specPaste, added: [], issues: [] });
    assert.equal(applyState(nothing, true, false), "disabled");
  });

  test("Retry of a failed Apply: sent, a line to fix, or nothing to add", () => {
    const valid = wrap({ ...specPaste, issues: [] });
    assert.equal(applyRetryBlock("a", valid, true), null);
    const invalid = applyRetryBlock("a", wrap(specPaste), true);
    assert.equal(invalid?.tone, "warning");
    assert.equal(invalid?.note.key, "settings.vocabulary.pasteBox.note.fixOne");
    const nothing = wrap({ ...specPaste, added: [], issues: [] });
    const empty = applyRetryBlock("a", nothing, true);
    assert.equal(empty?.tone, "info");
    assert.equal(
      empty?.note.key,
      "settings.vocabulary.pasteBox.note.nothingToAdd",
    );
    assert.equal(applyRetryBlock("a", null, false)?.tone, "warning");
  });
});

describe("paste preview", () => {
  test("rows follow input order and name the earlier spelling", () => {
    const rows = previewRows(specPaste);
    assert.deepEqual(
      rows.map((row) => [row.line, row.kind]),
      [
        [1, "add"],
        [2, "add"],
        [3, "duplicate"],
        [4, "add"],
        [5, "error"],
      ],
    );
    assert.equal(rows[2].sameAs, "Claude Code");
    assert.equal(rows[4].issue, "forbidden_character");
  });

  test("the summary counts each kind and warns while Apply is blocked", () => {
    assert.deepEqual(previewSummary(specPaste), {
      added: 3,
      duplicates: 1,
      errors: 1,
      tone: "warning",
    });
    assert.equal(previewSummary({ ...specPaste, issues: [] }).tone, "success");
    const full = { ...specPaste, issues: [], over_capacity: true };
    assert.equal(previewSummary(full).tone, "warning");
  });

  test("the note names the lines to fix, else why Apply is unavailable", () => {
    const blocked = applyNote("x", wrap(specPaste), true);
    assert.deepEqual(blocked, {
      key: "settings.vocabulary.pasteBox.note.fixOne",
      values: { lines: "5" },
    });
    const two = wrap({
      ...specPaste,
      issues: [
        { index: 4, issue: "forbidden_character" },
        { index: 6, issue: "empty" },
      ],
    });
    assert.equal(applyNote("x", two, true).values.lines, "5, 7");
    assert.match(applyNote("", null, false).key, /note\.empty$/);
    assert.match(applyNote("x", null, false).key, /note\.checking$/);
    const full = wrap({ ...specPaste, issues: [], over_capacity: true });
    assert.match(applyNote("x", full, true).key, /overCapacity$/);
    const nothing = wrap({ ...specPaste, added: [], issues: [] });
    assert.match(applyNote("x", nothing, true).key, /nothingToAdd$/);
  });

  test("the Apply label counts the terms to add", () => {
    assert.match(applyLabelKey(3, false), /applyMany$/);
    assert.match(applyLabelKey(1, false), /applyOne$/);
    assert.match(applyLabelKey(0, false), /applyZero$/);
    assert.match(applyLabelKey(3, true), /applying$/);
  });
});

describe("status", () => {
  test("each of the five statuses has its tone and wording", () => {
    const cases: Array<[Partial<VocabularyStatusReport>, string, string]> = [
      [{ status: "used_during_recognition" }, "success", "used.title"],
      [{ status: "not_supported" }, "neutral", "notSupported.title"],
      [{ status: "not_used_streaming" }, "neutral", "streaming.title"],
      [
        { status: "unknown_until_model_loads", reason: "model_not_loaded" },
        "info",
        "unknown.title",
      ],
      [
        { status: "no_term_sent", reason: "over_budget" },
        "warning",
        "noTermSent.title",
      ],
      [
        { status: "no_term_sent", reason: "empty_vocabulary" },
        "neutral",
        "noTermSent.title",
      ],
    ];
    for (const [overrides, tone, title] of cases) {
      assert.equal(statusTone(report(overrides)), tone);
      assert.equal(
        statusText(report(overrides)).title,
        `settings.vocabulary.status.${title}`,
      );
    }
  });

  test("bodies carry the reason of an unknown or empty status", () => {
    const unanswered = report({
      status: "unknown_until_model_loads",
      reason: "capability_query_unanswered",
    });
    assert.match(statusText(unanswered).body, /capability_query_unanswered$/);
    const invalid = report({
      status: "no_term_sent",
      reason: "invalid_terms_only",
    });
    assert.match(statusText(invalid).body, /invalid_terms_only$/);
  });

  test("sent and not-sent terms: the mockup's 42 of 45, grouped by reason", () => {
    const sent = Array.from({ length: 42 }, (_, index) => `Term ${index}`);
    const notSent = ["Jump Desktop", "iPhone Mirroring", "Screen Continuity"];
    const used = report({
      terms_sent: sent,
      terms_not_sent: notSent.map((term, offset) => ({
        index: 42 + offset,
        term,
        reason: "over_budget" as const,
      })),
    });
    assert.equal(sentTotal(used), 45);
    assert.match(statusText(used).body, /used\.sent$/);
    assert.deepEqual(notSentGroups(used.terms_not_sent), [
      { reason: "over_budget", terms: notSent },
    ]);
    assert.deepEqual([...overBudgetIndices(used)], [42, 43, 44]);
  });

  test("invalid legacy terms are grouped; no_context is never listed", () => {
    const groups = notSentGroups([
      { index: 0, term: "a<b", reason: "forbidden_character" },
      { index: 1, term: "dup", reason: "duplicate" },
      { index: 2, term: "x", reason: "too_long" },
      { index: 3, term: "Penflow", reason: "no_context" },
    ]);
    assert.deepEqual(groups, [
      { reason: "invalid", terms: ["a<b", "x"] },
      { reason: "duplicate", terms: ["dup"] },
    ]);
  });
});

describe("chips and refusals", () => {
  test("legacy issues and the model limit flag a chip", () => {
    const legacy = { index: 0, term: "a<b", issue: "forbidden_character" };
    assert.equal(
      chipFlagKey(legacy as never, false),
      "settings.vocabulary.flag.forbidden_character",
    );
    const fine = { index: 1, term: "OpenClaw", issue: null };
    assert.equal(
      chipFlagKey(fine, true),
      "settings.vocabulary.flag.over_budget",
    );
    assert.equal(chipFlagKey(fine, false), null);
  });

  test("a term refusal is explained by its reason, without Retry", () => {
    const duplicate = refusal({
      items: [{ index: 0, code: "term_duplicate", message: "Duplicate." }],
    });
    assert.equal(
      commitErrorKey(duplicate),
      "settings.vocabulary.errors.term_duplicate",
    );
    assert.equal(isTermRefusal(duplicate), true);
    assert.equal(canRetry(duplicate), false);
    const tooLong = refusal({
      items: [{ index: 0, code: "term_too_long", message: "Too long." }],
    });
    assert.equal(
      commitErrorKey(tooLong),
      "settings.vocabulary.errors.term_too_long",
    );
  });

  test("list refusals map to the vocabulary or shared settings wording", () => {
    const cases: Array<[Partial<SettingsCommandError>, string, boolean]> = [
      [{ code: "limit_exceeded" }, "settings.vocabulary.errors.limit", false],
      [
        { code: "revision_conflict" },
        "settings.vocabulary.errors.conflict",
        true,
      ],
      [{ code: "not_found" }, "settings.vocabulary.errors.notFound", false],
      [
        { code: "unavailable", outcome: "unknown" },
        "settings.vocabulary.errors.unknown",
        true,
      ],
      [
        { code: "persistence_failed" },
        "errors.settings.persistenceFailed",
        true,
      ],
      [
        { code: "internal_error", outcome: "committed" },
        "settings.vocabulary.errors.committed",
        false,
      ],
    ];
    for (const [overrides, key, retry] of cases) {
      assert.equal(commitErrorKey(refusal(overrides)), key);
      assert.equal(canRetry(refusal(overrides)), retry);
    }
    assert.equal(commitErrorKey(null), "errors.settings.generic");
    assert.equal(canRetry(null), true);
  });
});
