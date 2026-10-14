// Tests of the "Add shortcut" draft states (004 Behavioral AC has_validation,
// is_submittable, async_action); run with `bun run test:shortcut-states`.
import { describe, expect, test } from "bun:test";
import type { ShortcutEditError } from "@/bindings";
import {
  CLOSED_DRAFT,
  type DraftEvent,
  type DraftState,
  addControlState,
  canCommit,
  draftReducer,
  recorderValidity,
} from "./shortcut-draft-model";

const CONFLICT: ShortcutEditError = {
  code: "shortcut_conflict",
  conflict: {
    kind: "duplicate",
    owning_action: "transcribe_with_post_process",
    owning_shortcut: "option+shift+space",
  },
};
const SYSTEM_REFUSAL: ShortcutEditError = {
  code: "registration_failed",
  message: "HotKey already registered",
};

const run = (...events: DraftEvent[]): DraftState =>
  events.reduce(draftReducer, CLOSED_DRAFT);

const checked = (refusal: ShortcutEditError | null): DraftState =>
  run(
    { type: "open" },
    { type: "captured", shortcut: "shift+option+space" },
    { type: "checked", shortcut: "shift+option+space", refusal },
  );

describe("recorder validity", () => {
  test("nothing recorded: empty, Add unavailable", () => {
    const draft = run({ type: "open" });
    expect(recorderValidity(draft)).toBe("empty");
    expect(addControlState(draft, true, false)).toBe("disabled");
  });

  test("a conflicting combination is invalid and disables Add", () => {
    const draft = checked(CONFLICT);
    expect(recorderValidity(draft)).toBe("invalid");
    expect(draft.refusal).toEqual(CONFLICT);
    expect(addControlState(draft, true, false)).toBe("disabled");
  });

  test("a unique combination is valid and enables Add", () => {
    const draft = checked(null);
    expect(recorderValidity(draft)).toBe("valid");
    expect(addControlState(draft, true, false)).toBe("enabled");
  });

  test("while the check runs, Add stays unavailable", () => {
    const draft = run({ type: "open" }, { type: "captured", shortcut: "f13" });
    expect(draft.phase).toBe("checking");
    expect(addControlState(draft, true, false)).toBe("disabled");
  });
});

describe("draft life cycle", () => {
  test("Add opens the recorder only when the action has room", () => {
    expect(addControlState(CLOSED_DRAFT, true, false)).toBe("enabled");
    expect(addControlState(CLOSED_DRAFT, false, false)).toBe("disabled");
    expect(addControlState(CLOSED_DRAFT, true, true)).toBe("disabled");
  });

  test("leaving an empty recorder closes it; a captured draft stays", () => {
    expect(run({ type: "open" }, { type: "stopped" })).toEqual(CLOSED_DRAFT);
    const kept = draftReducer(checked(null), { type: "stopped" });
    expect(kept.phase).toBe("valid");
  });

  test("an answer for an older capture is ignored", () => {
    const draft = run(
      { type: "open" },
      { type: "captured", shortcut: "f13" },
      { type: "captured", shortcut: "f14" },
      { type: "checked", shortcut: "f13", refusal: CONFLICT },
    );
    expect(draft).toEqual({
      phase: "checking",
      shortcut: "f14",
      refusal: null,
    });
  });

  test("committing shows Add loading, then closes on success", () => {
    const committing = draftReducer(checked(null), { type: "commit" });
    expect(addControlState(committing, true, true)).toBe("loading");
    expect(draftReducer(committing, { type: "committed" })).toEqual(
      CLOSED_DRAFT,
    );
  });

  test("an invalid draft cannot be committed", () => {
    const invalid = checked(CONFLICT);
    expect(canCommit(invalid)).toBe(false);
    expect(draftReducer(invalid, { type: "commit" })).toEqual(invalid);
  });

  test("a system refusal keeps the draft and allows a retry", () => {
    const refused = run(
      { type: "open" },
      { type: "captured", shortcut: "cmd+shift+5" },
      { type: "checked", shortcut: "cmd+shift+5", refusal: null },
      { type: "commit" },
      { type: "commitFailed", refusal: SYSTEM_REFUSAL },
    );
    expect(refused.phase).toBe("invalid");
    expect(refused.shortcut).toBe("cmd+shift+5");
    expect(canCommit(refused)).toBe(true);
    expect(draftReducer(refused, { type: "commit" }).phase).toBe("committing");
  });

  test("a late answer never reopens a discarded draft", () => {
    const discarded = run(
      { type: "open" },
      { type: "captured", shortcut: "f13" },
      { type: "discard" },
      { type: "checked", shortcut: "f13", refusal: null },
    );
    expect(discarded).toEqual(CLOSED_DRAFT);
  });
});
