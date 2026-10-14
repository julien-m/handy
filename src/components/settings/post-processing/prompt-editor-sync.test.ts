/**
 * Synchronisation of the prompt editor with the library (005 AC-001): the
 * editor's own save against a library that lags one revision behind, a
 * failed reload, a change made elsewhere after the save, keystrokes typed
 * during a save, invalid prompts opened to be fixed and a creation answered
 * after the editor closed. Run with `bun run test:prompt-settings`.
 */
import { describe, expect, test } from "bun:test";
import type { ListedPrompt } from "@/bindings";
import type { CommitOutcome } from "./prompt-commands";
import type { EditorTarget, PromptDraft } from "./prompt-draft-model";
import {
  type EditorSync,
  createdIdToOpen,
  initialSyncOf,
  libraryReadingOf,
  reconcileWithLibrary,
  showsIssuesAtOpen,
  syncAfterSave,
} from "./prompt-editor-sync";

const listed = (patch: Partial<ListedPrompt> = {}): ListedPrompt => ({
  id: "p1",
  name: "Shorten",
  prompt: "Shorten ${output}",
  provider_override: null,
  starter: false,
  issues: [],
  shortcuts: [],
  readiness: {
    provider_id: "custom",
    model: "llama3.2",
    is_override: false,
    ready: true,
    missing: null,
  },
  ...patch,
});
const EDIT: EditorTarget = { mode: "edit", id: "p1" };
const FULL = { content: true, provider: true };
const before = listed();
const after = listed({ prompt: "Summarize ${output}" });

/** Opens `before`, types the new text and saves it at `revision`. */
const savedAt = (revision: number | null): EditorSync => {
  const opened = initialSyncOf(before);
  const typed = { ...opened.draft, text: "Summarize ${output}" };
  const editing = { ...opened, draft: typed };
  const result = { sent: typed, base: opened.saved, committed: FULL, revision };
  return syncAfterSave(editing, result);
};

// @spec AC-001: The editor's own save is never taken for a change elsewhere — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("own save against a library reload", () => {
  test("a library one revision behind neither rolls back nor reports a change", () => {
    const sync = savedAt(5);
    const stale = reconcileWithLibrary(sync, EDIT, {
      current: before,
      revision: 5,
    });
    expect(stale.external).toBe("none");
    expect(stale.sync).toBe(sync);
    expect(stale.sync.draft.text).toBe("Summarize ${output}");
    expect(stale.sync.saved.text).toBe("Summarize ${output}");
  });

  test("a failed reload keeps the saved values and the pending commit", () => {
    const sync = savedAt(5);
    // The library stays the one loaded before the save (revision 4).
    const kept = reconcileWithLibrary(sync, EDIT, {
      current: before,
      revision: 4,
    });
    expect(kept.sync.draft.text).toBe("Summarize ${output}");
    expect(kept.sync.pending?.revision).toBe(5);
  });

  test("the reload that lists the save settles it without a notice", () => {
    const confirmed = reconcileWithLibrary(savedAt(5), EDIT, {
      current: after,
      revision: 6,
    });
    expect(confirmed.external).toBe("none");
    expect(confirmed.sync.pending).toBeNull();
    expect(confirmed.sync.draft.text).toBe("Summarize ${output}");
  });

  test("a library listing the save is confirmed whatever its revision", () => {
    const pending = savedAt(null).pending;
    expect(libraryReadingOf(pending, { current: after, revision: null })).toBe(
      "confirmed",
    );
    expect(libraryReadingOf(pending, { current: before, revision: 9 })).toBe(
      "stale",
    );
  });

  test("a change made elsewhere after the save is taken when nothing is typed", () => {
    const other = listed({ prompt: "Translate ${output}" });
    const later = reconcileWithLibrary(savedAt(5), EDIT, {
      current: other,
      revision: 7,
    });
    expect(later.sync.draft.text).toBe("Translate ${output}");
    expect(later.sync.pending).toBeNull();
  });

  test("a change made elsewhere over typed text is reported, the text kept", () => {
    const sync = savedAt(5);
    const typing: EditorSync = {
      ...sync,
      draft: { ...sync.draft, text: "Mine ${output}" },
    };
    const other = listed({ prompt: "Translate ${output}" });
    const later = reconcileWithLibrary(typing, EDIT, {
      current: other,
      revision: 7,
    });
    expect(later.external).toBe("changed");
    expect(later.sync.draft.text).toBe("Mine ${output}");
  });

  test("a prompt deleted after the save is reported as deleted", () => {
    const gone = reconcileWithLibrary(savedAt(5), EDIT, {
      current: undefined,
      revision: 7,
    });
    expect(gone.external).toBe("deleted");
  });

  test("a prompt just created is not deleted before the library lists it", () => {
    const created = syncAfterSave(initialSyncOf(undefined), {
      sent: {
        name: "Summarize",
        text: after.prompt,
        provider: { providerId: "", model: "" },
      },
      base: initialSyncOf(undefined).saved,
      committed: FULL,
      revision: 3,
    });
    const waiting = reconcileWithLibrary(
      created,
      { mode: "edit", id: "new" },
      {
        current: undefined,
        revision: 3,
      },
    );
    expect(waiting.external).toBe("none");
  });
});

describe("save outcomes", () => {
  const opened = initialSyncOf(before);
  const sent: PromptDraft = { ...opened.draft, text: "Summarize ${output}" };

  test("keystrokes typed while the save runs are kept and stay unsaved", () => {
    const typedDuringSave = {
      ...opened,
      draft: { ...sent, text: "Summarize it ${output}" },
    };
    const next = syncAfterSave(typedDuringSave, {
      sent,
      base: opened.saved,
      committed: FULL,
      revision: 2,
    });
    expect(next.draft.text).toBe("Summarize it ${output}");
    expect(next.saved.text).toBe("Summarize ${output}");
  });

  test("a partial save keeps the form and records only what was stored", () => {
    const withOverride = {
      ...sent,
      provider: { providerId: "openai", model: "" },
    };
    const next = syncAfterSave(
      { ...opened, draft: withOverride },
      {
        sent: withOverride,
        base: opened.saved,
        committed: { content: true, provider: false },
        revision: 2,
      },
    );
    expect(next.draft).toBe(withOverride);
    expect(next.saved.text).toBe("Summarize ${output}");
    expect(next.saved.provider.providerId).toBe("");
  });

  test("a refusal that stored nothing changes nothing", () => {
    const editing = { ...opened, draft: sent };
    const next = syncAfterSave(editing, {
      sent,
      base: opened.saved,
      committed: { content: false, provider: false },
      revision: 2,
    });
    expect(next).toBe(editing);
  });
});

describe("opening and closing", () => {
  test("an invalid saved prompt shows its issues from the start", () => {
    const invalid = listed({ issues: ["name_too_long"] });
    expect(showsIssuesAtOpen(EDIT, invalid)).toBe(true);
    expect(showsIssuesAtOpen(EDIT, before)).toBe(false);
    expect(showsIssuesAtOpen({ mode: "new" }, undefined)).toBe(false);
  });

  test("a creation answered after Cancel never reopens the editor", () => {
    const outcome: CommitOutcome = { status: "saved", id: "p9" };
    expect(createdIdToOpen({ mode: "new" }, outcome, false)).toBeNull();
    expect(createdIdToOpen({ mode: "new" }, outcome, true)).toBe("p9");
    expect(createdIdToOpen(EDIT, outcome, true)).toBeNull();
  });
});
