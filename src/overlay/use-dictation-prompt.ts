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
  const { library, reload } = useLibrary(shown, view?.selectable ?? false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [feedbackKey, setFeedbackKey] = useState<string | null>(null);

  useEffect(() => {
    if (!recording || !view?.selectable) setMenuOpen(false);
  }, [recording, view?.selectable]);

  const select = useCallback(
    async (promptId: string | null) => {
      if (!view) return;
      const answer = await commands.selectDictationPrompt(
        view.session_id,
        promptId,
      );
      if (answer.status === "ok") {
        accept(answer.data);
        setFeedbackKey(null);
        return setMenuOpen(false);
      }
      setFeedbackKey(promptRefusalKey(answer.error));
      if (refusalClosesMenu(answer.error)) return setMenuOpen(false);
      await reload();
    },
    [view, accept, reload],
  );

  const openMenu = useCallback(() => {
    setFeedbackKey(null);
    setMenuOpen(true);
  }, []);
  const closeMenu = useCallback(() => setMenuOpen(false), []);
  const choose = useCallback(
    (promptId: string | null) =>
      void select(promptId).catch((error: unknown) => {
        console.warn("Indicator prompt choice failed", error);
        setFeedbackKey("overlay.prompt.choiceFailed");
      }),
    [select],
  );
  return {
    view,
    library,
    menuOpen,
    feedbackKey,
    openMenu,
    closeMenu,
    select: choose,
  };
};
