/**
 * Pure reading of a bulk or import preview (feature 002, AC-003/AC-005) and of rule-file
 * refusals: counts, Apply availability, row identities and translation keys. The preview itself
 * is always computed by `preview_word_replacements`.
 */
import type {
  ErrorItem,
  IgnoredLine,
  PreviewLine,
  PreviewStatus,
  ReplacementPreview,
  ReplacementRule,
  SettingsCommandError,
} from "@/bindings";
import {
  SETTINGS_ERROR_KEYS,
  settingsErrorKey,
} from "@/stores/settings-errors";

/** Rows of a preview by outcome. */
export interface PreviewCounts {
  readonly adds: number;
  readonly updates: number;
  readonly deletes: number;
  readonly unchanged: number;
  readonly errors: number;
}

/**
 * Counts of a preview; errors use the backend counter, which includes unlisted error lines.
 * @param preview preview of the input
 * @returns rows by outcome
 */
export const previewCounts = (preview: ReplacementPreview): PreviewCounts => {
  const count = (status: PreviewStatus): number =>
    preview.lines.filter((line) => line.status === status).length;
  return {
    adds: count("add"),
    updates: count("update"),
    deletes: count("delete"),
    unchanged: count("unchanged"),
    errors: preview.error_lines,
  };
};

/**
 * Whether Apply may commit the previewed set: no error line and no whole-set issue (AC-003).
 * A preview equal to the saved set has nothing to apply.
 * @param preview preview of the input
 * @returns true when Apply is available
 */
export const canApplyPreview = (preview: ReplacementPreview): boolean =>
  preview.error_lines === 0 &&
  preview.set_issues.length === 0 &&
  !preview.is_noop;

/**
 * Whether the preview has an error that blocks Apply.
 * @param preview preview of the input
 * @returns true for an error line or a whole-set issue
 */
export const hasPreviewErrors = (preview: ReplacementPreview): boolean =>
  preview.error_lines > 0 || preview.set_issues.length > 0;

/**
 * First listed error line, named by the apply bar ("Fix line 4 to apply").
 * @param preview preview of the input
 * @returns the 1-based line number, or null
 */
export const firstErrorLine = (preview: ReplacementPreview): number | null =>
  preview.lines.find((line) => line.status === "error")?.line_number ?? null;

/**
 * Stable row identity: the input line number, or the deleted rule's id for a deletion (which
 * has no input line).
 * @param line preview row
 * @returns a key unique within the preview
 */
export const previewLineKey = (line: PreviewLine): string =>
  line.line_number === null
    ? `delete-${line.rule_id ?? "unknown"}`
    : `line-${line.line_number}`;

/**
 * Translation key of a row's outcome badge.
 * @param status row outcome
 * @returns a key under `settings.replacements.preview.kinds`
 */
export const previewKindKey = (status: PreviewStatus): string =>
  `settings.replacements.preview.kinds.${status}`;

/**
 * Translation key of an ignored row.
 * @param reason why the line carries no rule
 * @returns a key under `settings.replacements.preview.ignored`
 */
export const ignoredKey = (reason: IgnoredLine | null): string =>
  `settings.replacements.preview.ignored.${reason ?? "blank"}`;

/** First change of an import, shown under its counts ("cloud code → Claude Code CLI"). */
export interface ImportExample {
  readonly status: "update" | "add" | "delete";
  readonly recognized: string;
  readonly replacement: string;
  /** 1-based position of the updated rule in the saved list, for an update. */
  readonly position: number | null;
}

/**
 * First update, else first addition, else first deletion of a preview.
 * @param preview import preview
 * @param saved saved rules, to name an update's kept position
 * @returns the example, or null when the import changes nothing
 */
export const importExample = (
  preview: ReplacementPreview,
  saved: ReadonlyArray<ReplacementRule>,
): ImportExample | null => {
  const order: ReadonlyArray<ImportExample["status"]> = [
    "update",
    "add",
    "delete",
  ];
  for (const status of order) {
    const line = preview.lines.find((row) => row.status === status);
    // The first matching outcome in this priority order is the most telling example.
    if (!line) continue;
    const index = saved.findIndex((rule) => rule.id === line.rule_id);
    return {
      status,
      recognized: line.recognized ?? "",
      replacement: line.replacement ?? "",
      position: status === "update" && index >= 0 ? index + 1 : null,
    };
  }
  return null;
};

/**
 * Number of lines of a file, as the import dialog names it; a final line break does not start
 * another line.
 * @param text file text
 * @returns the line count (0 for an empty file)
 */
export const lineCount = (text: string): number => {
  if (text.length === 0) return 0;
  const breaks = text.match(/\n/g)?.length ?? 0;
  return text.endsWith("\n") ? breaks : breaks + 1;
};

/** Why an import file was refused, each with its own alert. */
export type FileRefusal = "encoding" | "tooLarge" | "unreadable";

/**
 * Category of a refused import file.
 * @param error typed refusal of `import_word_replacements_file`
 * @returns the alert to show
 */
export const fileRefusal = (error: SettingsCommandError): FileRefusal => {
  if (error.reason === "invalid_encoding") return "encoding";
  if (error.code === "limit_exceeded") return "tooLarge";
  return "unreadable";
};

/**
 * Translation key of a refused export.
 * @param error typed refusal of `export_word_replacements_file`
 * @returns a key under `settings.replacements.export`, or a shared settings error key
 */
export const exportFailureKey = (error: SettingsCommandError): string => {
  switch (error.detail_reason) {
    case "export_symbolic_link":
      return "settings.replacements.export.symbolicLink";
    case "export_read_only":
      return "settings.replacements.export.readOnly";
    case "export_not_written":
      return "settings.replacements.export.notWritten";
    default:
      return settingsErrorKey(error);
  }
};

/**
 * Translation key of a refused commit (Save, Apply or import), never parsed from the message.
 * @param error typed refusal
 * @returns a shared settings error key
 */
export const commitFailureKey = (error: SettingsCommandError): string =>
  error.code === "validation_failed"
    ? "settings.replacements.commit.invalid"
    : settingsErrorKey(error);

/** A refused or failed commit, as the editor shows it; the saved rules are unchanged unless
 * `uncertain`. */
export interface CommitFailure {
  /** Translation key of the message. */
  readonly key: string;
  /** The rules moved on since the editor read them: Reload, never a silent overwrite. */
  readonly conflict: boolean;
  /** The commit may have been stored (a live effect failed, or the outcome is unknown). */
  readonly uncertain: boolean;
  /** Refused positions (draft rows or input lines), from `details.items`. */
  readonly items: ReadonlyArray<ErrorItem>;
}

/**
 * Editor view of a typed commit refusal.
 * @param error typed refusal of Save, Apply or an import
 * @returns the failure to show
 */
export const commitFailure = (error: SettingsCommandError): CommitFailure => ({
  key: commitFailureKey(error),
  conflict: error.code === "revision_conflict",
  uncertain: error.outcome === "committed" || error.outcome === "unknown",
  items: error.items ?? [],
});

/** Failure of a command that threw instead of returning a typed refusal. */
export const UNEXPECTED_FAILURE: CommitFailure = {
  key: SETTINGS_ERROR_KEYS.generic,
  conflict: false,
  uncertain: true,
  items: [],
};
