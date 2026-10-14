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
  /** Settings revision the shown library was loaded for. */
  loadedRevision: number | null;
  loadState: LibraryLoadState;
  reload: () => void;
}

interface Loaded {
  library: PromptLibraryView;
  revision: number | null;
}

/**
 * Reads the library once for `revision`.
 * @returns The library, or null when it could not be read (logged).
 */
const loadLibrary = async (revision: number | null): Promise<Loaded | null> => {
  try {
    const result = await commands.getPromptLibrary();
    if (result.status === "ok") return { library: result.data, revision };
    console.error("Failed to load the prompt library:", result.error.code);
  } catch (failure) {
    console.error("Failed to load the prompt library:", failure);
  }
  return null;
};

/**
 * Loads the library on mount and whenever the settings revision changes.
 * A failed reload keeps the last library shown and reports the error.
 */
export const usePromptLibrary = (): PromptLibraryState => {
  const revision = useSettingsStore((state) => state.revision);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [loadState, setLoadState] = useState<LibraryLoadState>("loading");
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let isCurrent = true;
    const load = async () => {
      const next = await loadLibrary(revision);
      if (!isCurrent) return;
      if (next) setLoaded(next);
      setLoadState(next ? "ready" : "error");
    };
    void load();
    return () => {
      isCurrent = false;
    };
  }, [revision, attempt]);
  const reload = useCallback(() => {
    setLoadState("loading");
    setAttempt((count) => count + 1);
  }, []);
  const loadedRevision = loaded?.revision ?? null;
  return {
    library: loaded?.library ?? null,
    loadedRevision,
    loadState,
    reload,
  };
};
