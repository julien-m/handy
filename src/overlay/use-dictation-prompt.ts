// Prompt of the dictation in progress for the recording indicator (feature 005): follows
// `dictation-prompt-changed`, reads the library for the menu and applies a choice through
// `select_dictation_prompt`. A choice changes this dictation only; the default is untouched.
import { useCallback, useEffect, useState } from "react";
import {
  commands,
  events,
  type DictationPromptView,
  type PromptLibraryView,
} from "@/bindings";
import {
  newerPromptView,
  promptRefusalKey,
  refusalClosesMenu,
} from "./prompt-model";

/** Library state for the menu. */
export type LibraryState =
  | { status: "loading" }
  | { status: "ready"; library: PromptLibraryView }
  | { status: "error" };

/** What the indicator needs from the prompt session. */
export interface PromptControls {
  view: DictationPromptView | null;
  library: LibraryState;
  menuOpen: boolean;
  feedbackKey: string | null;
  openMenu: () => void;
  closeMenu: () => void;
  select: (promptId: string | null) => void;
}

const useLibrary = (shown: boolean, selectable: boolean) => {
  const [library, setLibrary] = useState<LibraryState>({ status: "loading" });
  const reload = useCallback(async () => {
    try {
      const answer = await commands.getPromptLibrary();
      setLibrary(
        answer.status === "ok"
          ? { status: "ready", library: answer.data }
          : { status: "error" },
      );
    } catch (error) {
      console.warn("Could not read the prompt library", error);
      setLibrary({ status: "error" });
    }
  }, []);
  useEffect(() => {
    if (shown && selectable) void reload();
  }, [shown, selectable, reload]);
  return { library, reload };
};

const useView = (shown: boolean) => {
  const [view, setView] = useState<DictationPromptView | null>(null);
  const accept = useCallback((incoming: DictationPromptView | null) => {
    setView((current) => newerPromptView(current, incoming));
  }, []);
  useEffect(() => {
    const unlisten = events.dictationPromptChanged.listen((event) =>
      accept(event.payload.prompt),
    );
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [accept]);
  useEffect(() => {
    if (!shown) return;
    commands.getDictationPrompt().then(accept, (error: unknown) => {
      // The `dictation-prompt-changed` event still updates the pill.
      console.warn("Could not read the dictation prompt", error);
    });
  }, [shown, accept]);
  return { view, accept };
};

/** Open state of the prompt menu and the feedback shown in it. */
const useMenu = (recording: boolean, selectable: boolean) => {
  const [menuOpen, setMenuOpen] = useState(false);
  const [feedbackKey, setFeedbackKey] = useState<string | null>(null);
  useEffect(() => {
    // Choices end with the recording, and need a selectable prompt.
    if (!recording || !selectable) setMenuOpen(false);
  }, [recording, selectable]);
  const openMenu = useCallback(() => {
    setFeedbackKey(null);
    setMenuOpen(true);
  }, []);
  const closeMenu = useCallback(() => setMenuOpen(false), []);
  return { menuOpen, feedbackKey, setFeedbackKey, openMenu, closeMenu };
};

type PromptMenu = ReturnType<typeof useMenu>;

/**
 * Applies a menu choice through `select_dictation_prompt`: an accepted choice updates the pill
 * and closes the menu; a refusal is explained, and either closes the menu or rereads the
 * library so the menu shows what can still be chosen.
 */
const useSelect = (
  view: DictationPromptView | null,
  accept: (incoming: DictationPromptView | null) => void,
  reload: () => Promise<void>,
  menu: PromptMenu,
) => {
  const { setFeedbackKey, closeMenu } = menu;
  return useCallback(
    async (promptId: string | null) => {
      if (!view) return;
      const answer = await commands.selectDictationPrompt(
        view.session_id,
        promptId,
      );
      if (answer.status === "ok") {
        accept(answer.data);
        setFeedbackKey(null);
        return closeMenu();
      }
      setFeedbackKey(promptRefusalKey(answer.error));
      if (refusalClosesMenu(answer.error)) return closeMenu();
      await reload();
    },
    [view, accept, reload, setFeedbackKey, closeMenu],
  );
};

// @spec FR-002: Choice for this dictation only — .specs/features/005-custom-ai-prompts/spec.md#fr-002
/**
 * Prompt pill state and actions; `shown` is true while a dictation shows the overlay and
 * `recording` while choices are still accepted (before the stop).
 */
export const useDictationPrompt = (
  shown: boolean,
  recording: boolean,
): PromptControls => {
  const { view, accept } = useView(shown);
  const selectable = view?.selectable ?? false;
  const { library, reload } = useLibrary(shown, selectable);
  const menu = useMenu(recording, selectable);
  const select = useSelect(view, accept, reload, menu);
  const { setFeedbackKey } = menu;
  const choose = useCallback(
    (promptId: string | null) =>
      void select(promptId).catch((error: unknown) => {
        console.warn("Indicator prompt choice failed", error);
        setFeedbackKey("overlay.prompt.choiceFailed");
      }),
    [select, setFeedbackKey],
  );
  const { menuOpen, feedbackKey, openMenu, closeMenu } = menu;
  const menuControls = { menuOpen, feedbackKey, openMenu, closeMenu };
  return { view, library, ...menuControls, select: choose };
};
