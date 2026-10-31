/**
 * Pure rules of the Quick Add window (018): which tab opens, when Add may dispatch, and the
 * translated reasons of a refused addition (the 002 and 003 keys, never new wording).
 */
import type { QuickAddKind, SettingsCommandError } from "@/bindings";
import { itemKey } from "@/components/settings/word-replacements/rules-model";
import { commitFailure } from "@/components/settings/word-replacements/preview-model";
import { commitErrorKey } from "@/components/settings/vocabulary/vocabulary-model";
import { SETTINGS_ERROR_KEYS } from "@/stores/settings-errors";

/** Test-id prefix of the window (`spec.md` screen `quick-add`). */
export const QUICK_ADD_SCREEN = "quick-add";

/** Event switching an open window to another kind (`quick_add::QUICK_ADD_KIND_EVENT`). */
export const QUICK_ADD_KIND_EVENT = "quick-add-kind";

/** Text typed in the window. */
export interface QuickAddValues {
  heard: string;
  written: string;
  term: string;
}

// @spec AC-007: The confirmation goes away by itself — .specs/features/018-quick-add-from-tray/spec.md#ac-007
/** How long the confirmation of a saved addition stays shown. */
export const ADDED_NOTICE_MS = 4000;

export const EMPTY_VALUES: QuickAddValues = {
  heard: "",
  written: "",
  term: "",
};

/**
 * Tab the window opens on, from its page query (`?kind=…`); a rule by default.
 * @param search `window.location.search`
 * @returns the kind named by the query, or `replacement`
 */
export const parseKind = (search: string): QuickAddKind =>
  new URLSearchParams(search).get("kind") === "vocabulary"
    ? "vocabulary"
    : "replacement";

// @spec AC-008: Add unavailable while empty or busy — .specs/features/018-quick-add-from-tray/spec.md#ac-008
/**
 * Whether Add may dispatch: every field of the kind holds text and no commit is in flight; the
 * backend validator judges the rest.
 * @param kind current tab
 * @param values typed text
 * @param busy a commit is in flight
 * @returns true when Add is enabled
 */
export const canSubmit = (
  kind: QuickAddKind,
  values: QuickAddValues,
  busy: boolean,
): boolean => {
  if (busy) return false;
  if (kind === "vocabulary") return values.term.trim() !== "";
  return values.heard.trim() !== "" && values.written.trim() !== "";
};

/** A refused or failed addition, as the window shows it. */
export interface QuickAddFailure {
  /** Translation keys, one per distinct reason. */
  readonly keys: ReadonlyArray<string>;
  /** "Nothing saved" reminder of the kind; null when the addition may have been stored. */
  readonly unchangedKey: string | null;
}

/** "Nothing saved" reminder of each kind (002, 003 wording). */
const UNCHANGED_KEYS: Record<QuickAddKind, string> = {
  replacement: "settings.replacements.commit.unchanged",
  vocabulary: "settings.vocabulary.errors.saved",
};

// @spec AC-005: 002 reasons of a refused rule — .specs/features/018-quick-add-from-tray/spec.md#ac-005
// @spec AC-006: 003 refusal of a term — .specs/features/018-quick-add-from-tray/spec.md#ac-006
/**
 * Reasons of a refused addition, from the typed refusal; a failure without one is generic.
 * @param kind tab that committed
 * @param error typed refusal, or null when the command threw
 * @returns the keys to show and the "nothing saved" reminder, absent when uncertain
 */
export const quickAddFailure = (
  kind: QuickAddKind,
  error: SettingsCommandError | null,
): QuickAddFailure => {
  const uncertain =
    !error || error.outcome === "committed" || error.outcome === "unknown";
  const unchangedKey = uncertain ? null : UNCHANGED_KEYS[kind];
  if (kind === "vocabulary") {
    return { keys: [commitErrorKey(error)], unchangedKey };
  }
  if (!error) return { keys: [SETTINGS_ERROR_KEYS.generic], unchangedKey };
  const items = error.items ?? [];
  const keys =
    error.code === "validation_failed" && items.length > 0
      ? [...new Set(items.map(itemKey))]
      : [commitFailure(error).key];
  return { keys, unchangedKey };
};
