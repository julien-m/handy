// Pure rules of the prompt pill and menu on the recording indicator (feature 005): when the
// pill shows, which rows the menu lists, which prompt is current and how a refused choice is
// explained. The indicator only ever shows the view the backend acknowledged.
import type {
  DictationPromptView,
  ListedPrompt,
  PromptChoiceRefusal,
  PromptLibraryView,
} from "@/bindings";
import { formatKeyCombination, type OSType } from "@/lib/utils/keyboard";

/** One row of the indicator prompt menu; `id === null` is "None (no AI processing)". */
export interface PromptMenuRow {
  id: string | null;
  name: string | null;
  shortcut: string | null;
  current: boolean;
  usable: boolean;
}

const MAC_MODIFIERS: Record<string, string> = {
  ctrl: "⌃",
  control: "⌃",
  option: "⌥",
  alt: "⌥",
  shift: "⇧",
  command: "⌘",
  cmd: "⌘",
  meta: "⌘",
  super: "⌘",
};

const sideless = (part: string): string =>
  part.trim().replace(/_(left|right)$/, "");

/**
 * Short label of a stored shortcut for the overlay badges: modifier glyphs on macOS ("⌃ ⌥ M"),
 * Handy's own wording elsewhere ("Ctrl + Alt + M").
 */
export const compactShortcut = (binding: string, osType: OSType): string => {
  if (osType !== "macos") return formatKeyCombination(binding, osType);
  return binding
    .split("+")
    .map(
      (part) =>
        MAC_MODIFIERS[sideless(part)] ?? formatKeyCombination(part, osType),
    )
    .filter((label) => label.length > 0)
    .join(" ");
};

/** True when the indicator shows the prompt pill: a recording with AI processing on. */
export const pillVisible = (
  view: DictationPromptView | null,
  recording: boolean,
): view is DictationPromptView => view !== null && view.selectable && recording;

/** The listed prompt with `id`, if the library has it. */
export const findPrompt = (
  library: PromptLibraryView | null,
  id: string | null,
): ListedPrompt | null =>
  id === null ? null : (library?.prompts.find((p) => p.id === id) ?? null);

/** The first shortcut of a prompt, formatted for the overlay, or `null` without one. */
export const promptShortcut = (
  prompt: ListedPrompt | null,
  osType: OSType,
): string | null => {
  const first = prompt?.shortcuts.find((shortcut) => shortcut.length > 0);
  return first ? compactShortcut(first, osType) : null;
};

// @spec AC-003: Indicator prompt menu, None and every prompt — .specs/features/005-custom-ai-prompts/spec.md#ac-003
/** Rows of the indicator menu: "None" first, then every prompt of the library in order. */
export const promptMenuRows = (
  library: PromptLibraryView,
  view: DictationPromptView,
  osType: OSType,
): PromptMenuRow[] => [
  {
    id: null,
    name: null,
    shortcut: null,
    current: view.prompt_id === null,
    usable: true,
  },
  ...library.prompts.map((prompt) => ({
    id: prompt.id,
    name: prompt.name,
    shortcut: promptShortcut(prompt, osType),
    current: prompt.id === view.prompt_id,
    usable: prompt.issues.length === 0,
  })),
];

/** Name of the default prompt (the menu note), or `null` when the default is None. */
export const defaultPromptName = (library: PromptLibraryView): string | null =>
  findPrompt(library, library.default_prompt_id)?.name ?? null;

/**
 * Keep the newest acknowledged view: a view of an older session never replaces a newer one,
 * and `null` (no dictation) always clears it.
 */
export const newerPromptView = (
  current: DictationPromptView | null,
  incoming: DictationPromptView | null,
): DictationPromptView | null => {
  if (incoming === null || current === null) return incoming;
  return incoming.session_id < current.session_id ? current : incoming;
};

const REFUSAL_KEYS: Record<PromptChoiceRefusal, string> = {
  session_ended: "overlay.prompt.refused.sessionEnded",
  processing_off: "overlay.prompt.refused.processingOff",
  prompt_missing: "overlay.prompt.refused.promptMissing",
  prompt_not_usable: "overlay.prompt.refused.promptNotUsable",
};

/** Translation key explaining a refused prompt choice (nothing changed). */
export const promptRefusalKey = (refusal: PromptChoiceRefusal): string =>
  REFUSAL_KEYS[refusal];

/** True when a refusal means the menu cannot be used any more for this dictation. */
export const refusalClosesMenu = (refusal: PromptChoiceRefusal): boolean =>
  refusal === "session_ended" || refusal === "processing_off";
