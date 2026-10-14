/**
 * Binds the vocabulary editor store to the running app: rereads the list, the status and the
 * paste preview whenever the settings revision changes (this window, the CLI or a model switch),
 * rereads the status when a model loads or unloads, and previews the paste text after typing
 * pauses.
 */
import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";
import {
  createVocabularyStore,
  type VocabularyStore,
} from "../stores/vocabulary-store";
import { useSettingsStore } from "../stores/settingsStore";

/** Pause after the last keystroke before the paste preview is asked again (one call, not one per key). */
export const PREVIEW_DELAY_MS = 200;

/**
 * Reread everything a settings revision can change: the list, the status and, while the paste box
 * is open, its preview.
 * @param store editor store
 */
export const refreshVocabulary = (store: VocabularyStore): void => {
  const state = store.getState();
  void state.loadList();
  void state.loadStatus();
  if (state.pasteOpen) void state.loadPreview();
};

/**
 * Reread the status whenever a model loads or unloads.
 * @param store editor store
 * @returns a function that stops listening
 */
export const watchModelState = (store: VocabularyStore): (() => void) => {
  const unlisten = listen("model-state-changed", () => {
    void store.getState().loadStatus();
  });
  return () => {
    void unlisten.then((stop) => stop());
  };
};

/**
 * Ask for the paste preview once typing pauses; a new keystroke cancels the pending request.
 * @param store editor store
 * @param delayMs pause before the request
 * @returns a function that cancels the pending request
 */
export const schedulePreview = (
  store: VocabularyStore,
  delayMs = PREVIEW_DELAY_MS,
): (() => void) => {
  const timer = setTimeout(() => {
    void store.getState().loadPreview();
  }, delayMs);
  return () => clearTimeout(timer);
};

/** Ask for the paste preview once typing pauses. */
const usePreviewDebounce = (store: VocabularyStore) => {
  const pasteOpen = useStore(store, (state) => state.pasteOpen);
  const pasteText = useStore(store, (state) => state.pasteText);
  useEffect(() => {
    if (!pasteOpen) return;
    return schedulePreview(store);
  }, [store, pasteOpen, pasteText]);
};

/**
 * Vocabulary editor store for one mounted editor, kept in sync with the backend.
 * @returns the store; its state is read with `useStore(store, selector)`
 */
export const useVocabulary = (): VocabularyStore => {
  const [store] = useState(() => createVocabularyStore(commands));
  const revision = useSettingsStore((state) => state.revision);
  useEffect(() => refreshVocabulary(store), [store, revision]);
  useEffect(() => watchModelState(store), [store]);
  usePreviewDebounce(store);
  return store;
};
