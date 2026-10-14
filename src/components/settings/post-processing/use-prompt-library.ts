// The prompt library as the backend publishes it (prompts with their issues,
// shortcuts and readiness, and the default prompt), reloaded after every
// committed settings revision so a change made elsewhere (CLI, another
// window, a repair) shows here.
import { useCallback, useEffect, useState } from "react";
import { commands } from "@/bindings";
import type { PromptLibraryView } from "@/bindings";
import { useSettingsStore } from "../../../stores/settingsStore";

/** Load state of the library. */
export type LibraryLoadState = "loading" | "ready" | "error";

/** The library and how to reload it. */
export interface PromptLibraryState {
  library: PromptLibraryView | null;
  loadState: LibraryLoadState;
  reload: () => void;
}

/**
 * Loads the library on mount and whenever the settings revision changes.
 * A failed reload keeps the last library shown and reports the error.
 */
export const usePromptLibrary = (): PromptLibraryState => {
  const revision = useSettingsStore((state) => state.revision);
  const [library, setLibrary] = useState<PromptLibraryView | null>(null);
  const [loadState, setLoadState] = useState<LibraryLoadState>("loading");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let isCurrent = true;
    const load = async () => {
      const result = await commands.getPromptLibrary();
      if (!isCurrent) return;
      if (result.status === "ok") {
        setLibrary(result.data);
        setLoadState("ready");
      } else {
        console.error("Failed to load the prompt library:", result.error.code);
        setLoadState("error");
      }
    };
    load().catch((failure) => {
      console.error("Failed to load the prompt library:", failure);
      if (isCurrent) setLoadState("error");
    });
    return () => {
      isCurrent = false;
    };
  }, [revision, attempt]);
  const reload = useCallback(() => {
    setLoadState("loading");
    setAttempt((count) => count + 1);
  }, []);
  return { library, loadState, reload };
};
