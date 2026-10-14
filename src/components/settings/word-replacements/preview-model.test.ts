/**
 * Preview reading of the replacement editor (feature 002, AC-003/AC-005): counts, Apply
 * availability, row keys, the import example, file line counts and typed file refusals.
 * Run with `bun run test:word-replacements`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import type {
  PreviewLine,
  ReplacementPreview,
  SettingsCommandError,
} from "@/bindings";
import {
  canApplyPreview,
  commitFailure,
  exportFailureKey,
  fileRefusal,
  firstErrorLine,
  hasPreviewErrors,
  importExample,
  lineCount,
  previewCounts,
  previewLineKey,
} from "./preview-model";

const line = (overrides: Partial<PreviewLine>): PreviewLine => ({
  line_number: 1,
  status: "add",
  ignored: null,
  rule_id: null,
  recognized: null,
  replacement: null,
  issues: [],
  duplicate_lines: [],
  ...overrides,
});

const preview = (
  lines: PreviewLine[],
  overrides: Partial<ReplacementPreview> = {},
): ReplacementPreview => ({
  mode: "replace",
  input_sha256: "0",
  lines,
  set_issues: [],
  candidate: [],
  is_noop: false,
  ignored_lines: lines.filter((row) => row.status === "ignored").length,
  error_lines: lines.filter((row) => row.status === "error").length,
  omitted_lines: 0,
  ...overrides,
});

// The bulk mockup: header, unchanged, update, missing separator, add.
const bulk = preview([
  line({ line_number: 1, status: "ignored", ignored: "header" }),
  line({ line_number: 2, status: "unchanged", rule_id: "a" }),
  line({ line_number: 3, status: "update", rule_id: "b" }),
  line({
    line_number: 4,
    status: "error",
    issues: [{ field: "line", kind: "missing_separator" }],
  }),
  line({ line_number: 5, status: "add", recognized: "Open Claw" }),
]);

const refusal = (
  overrides: Partial<SettingsCommandError>,
): SettingsCommandError => ({
  code: "validation_failed",
  message: "Setting value failed validation.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: null,
  ...overrides,
});

describe("preview", () => {
  test("an error line disables Apply and names its line", () => {
    assert.deepEqual(previewCounts(bulk), {
      adds: 1,
      updates: 1,
      deletes: 0,
      unchanged: 1,
      errors: 1,
    });
    assert.equal(hasPreviewErrors(bulk), true);
    assert.equal(canApplyPreview(bulk), false);
    assert.equal(firstErrorLine(bulk), 4);
  });

  test("a valid change can be applied; a no-op or a set issue cannot", () => {
    const valid = preview([line({ status: "add" })]);
    assert.equal(canApplyPreview(valid), true);
    assert.equal(canApplyPreview({ ...valid, is_noop: true }), false);
    const tooMany = { ...valid, set_issues: ["too_many_rules" as const] };
    assert.equal(hasPreviewErrors(tooMany), true);
    assert.equal(canApplyPreview(tooMany), false);
  });

  test("unlisted error lines still count", () => {
    const counted = preview([], { error_lines: 1500, omitted_lines: 500 });
    assert.equal(previewCounts(counted).errors, 1500);
    assert.equal(canApplyPreview(counted), false);
  });

  test("deletions are keyed by rule id, other rows by line", () => {
    assert.equal(
      previewLineKey(
        line({ line_number: null, status: "delete", rule_id: "z" }),
      ),
      "delete-z",
    );
    assert.equal(previewLineKey(line({ line_number: 7 })), "line-7");
  });
});

describe("import example", () => {
  const saved = [
    { id: "a", recognized: "Cloud Code", replacement: "Claude Code" },
    { id: "b", recognized: "Open Claw", replacement: "OpenClaw" },
  ];

  test("an update names the position it keeps", () => {
    const merged = preview([
      line({
        line_number: 1,
        status: "add",
        recognized: "x",
        replacement: "y",
      }),
      line({
        line_number: 2,
        status: "update",
        rule_id: "a",
        recognized: "cloud code",
        replacement: "Claude Code CLI",
      }),
    ]);
    assert.deepEqual(importExample(merged, saved), {
      status: "update",
      recognized: "cloud code",
      replacement: "Claude Code CLI",
      position: 1,
    });
  });

  test("otherwise the first addition, else nothing", () => {
    const added = preview([
      line({ status: "add", recognized: "Open Claw", replacement: "OpenClaw" }),
    ]);
    assert.equal(importExample(added, saved)?.status, "add");
    assert.equal(importExample(added, saved)?.position, null);
    assert.equal(importExample(preview([]), saved), null);
  });
});

describe("files", () => {
  test("line count ignores a final line break", () => {
    assert.equal(lineCount(""), 0);
    assert.equal(lineCount("a;b"), 1);
    assert.equal(lineCount("a;b\n"), 1);
    assert.equal(lineCount("a;b\r\nc;d\r\n"), 2);
    assert.equal(lineCount("h\n\na;b"), 3);
  });

  test("refused imports: encoding, size, then unreadable", () => {
    assert.equal(
      fileRefusal(refusal({ reason: "invalid_encoding" })),
      "encoding",
    );
    assert.equal(fileRefusal(refusal({ code: "limit_exceeded" })), "tooLarge");
    assert.equal(fileRefusal(refusal({ code: "usage_error" })), "unreadable");
  });

  test("refused exports keep their typed reason", () => {
    const failed = (reason: SettingsCommandError["detail_reason"]) =>
      exportFailureKey(
        refusal({ code: "persistence_failed", detail_reason: reason }),
      );
    assert.equal(
      failed("export_symbolic_link"),
      "settings.replacements.export.symbolicLink",
    );
    assert.equal(
      failed("export_read_only"),
      "settings.replacements.export.readOnly",
    );
    assert.equal(
      failed("export_not_written"),
      "settings.replacements.export.notWritten",
    );
  });
});

describe("commit failures", () => {
  test("a revision conflict asks for a reload and changed nothing", () => {
    const failure = commitFailure(
      refusal({ code: "revision_conflict", current_revision: 9 }),
    );
    assert.equal(failure.conflict, true);
    assert.equal(failure.uncertain, false);
  });

  test("an invalid draft keeps its located items", () => {
    const failure = commitFailure(
      refusal({
        reason: "invalid_items",
        items: [{ index: 2, code: "duplicate_recognized", message: "dup" }],
      }),
    );
    assert.equal(failure.key, "settings.replacements.commit.invalid");
    assert.equal(failure.items[0].index, 2);
  });

  test("a committed or unknown outcome is uncertain", () => {
    assert.equal(
      commitFailure(refusal({ code: "internal_error", outcome: "committed" }))
        .uncertain,
      true,
    );
    assert.equal(
      commitFailure(refusal({ code: "unavailable", outcome: "unknown" }))
        .uncertain,
      true,
    );
  });
});
