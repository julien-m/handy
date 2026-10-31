import { describe, expect, test } from "bun:test";
import type { SettingsCommandError } from "@/bindings";
import {
  EMPTY_VALUES,
  canSubmit,
  parseKind,
  quickAddFailure,
} from "./quick-add-model";

const refusal = (fields: Partial<SettingsCommandError>): SettingsCommandError =>
  ({
    code: "validation_failed",
    message: "",
    outcome: null,
    items: [],
    ...fields,
  }) as unknown as SettingsCommandError;

describe("Quick Add model (018)", () => {
  test("the page query selects the tab, a rule by default", () => {
    expect(parseKind("?kind=vocabulary")).toBe("vocabulary");
    expect(parseKind("?kind=replacement")).toBe("replacement");
    expect(parseKind("")).toBe("replacement");
  });

  test("Add needs every field of the tab and no commit in flight (AC-008)", () => {
    const rule = {
      ...EMPTY_VALUES,
      heard: "cloud code",
      written: "Claude Code",
    };
    expect(canSubmit("replacement", rule, false)).toBe(true);
    expect(canSubmit("replacement", { ...rule, written: "  " }, false)).toBe(
      false,
    );
    expect(canSubmit("replacement", rule, true)).toBe(false);
    expect(canSubmit("vocabulary", rule, false)).toBe(false);
    expect(
      canSubmit("vocabulary", { ...EMPTY_VALUES, term: "Kubernetes" }, false),
    ).toBe(true);
  });

  test("a refused rule shows each distinct 002 reason once and says nothing was saved (AC-005)", () => {
    const failure = quickAddFailure(
      "replacement",
      refusal({
        items: [
          { index: 3, code: "duplicate_recognized", message: "" },
          { index: 3, code: "duplicate_recognized", message: "" },
        ],
      }),
    );
    expect(failure.keys).toEqual(["settings.replacements.issues.duplicate"]);
    expect(failure.unchangedKey).toBe("settings.replacements.commit.unchanged");
  });

  test("a refused term shows the 003 reason (AC-006)", () => {
    const failure = quickAddFailure(
      "vocabulary",
      refusal({ items: [{ index: 0, code: "term_duplicate", message: "" }] }),
    );
    expect(failure.keys).toEqual(["settings.vocabulary.errors.term_duplicate"]);
    expect(failure.unchangedKey).toBe("settings.vocabulary.errors.saved");
  });

  test("an uncertain outcome never claims nothing was saved", () => {
    const committed = refusal({
      code: "persistence_failed",
      outcome: "committed",
    });
    expect(quickAddFailure("replacement", committed).unchangedKey).toBeNull();
    expect(quickAddFailure("vocabulary", null).unchangedKey).toBeNull();
  });
});
