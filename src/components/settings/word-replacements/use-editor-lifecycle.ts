/**
 * Lifecycle of the replacement editor while it is mounted: read the saved rules, follow every
 * committed settings change through `settings-changed` (any key: the base revision of a Save is
 * the global settings revision), keep an untouched text view on the saved rules, and register the
 * unsaved-edit guard of the settings window's navigation.
 */
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { registerUnsavedChangesGuard } from "@/stores/unsaved-changes";
import { isListDirty, useReplacementEditor } from "./editor-store";
import { isTextDirty, useReplacementText } from "./text-store";
import { useReplacementFiles } from "./file-store";

/**
 * Whether the editor's current view holds unsaved edits.
 * @returns true for a changed list draft or a changed text
 */
export const isEditorDirty = (): boolean => {
  const editor = useReplacementEditor.getState();
  return editor.view === "list"
    ? isListDirty(editor)
    : isTextDirty(useReplacementText.getState());
};

/**
 * Whether a Save, an Apply or an import Apply is in flight. The request owns the rules until it
 * settles: the guarded transitions (List/Text, Import…, section navigation) are not offered and
 * nothing is discarded meanwhile.
 * @returns true while a commit runs
 */
export const isCommitInFlight = (): boolean =>
  useReplacementEditor.getState().commit === "saving" ||
  useReplacementText.getState().applying ||
  useReplacementFiles.getState().phase === "applying";

/**
 * Reactive `isCommitInFlight`, for the controls it makes inactive.
 * @returns true while a commit runs
 */
export const useCommitInFlight = (): boolean => {
  const saving = useReplacementEditor((state) => state.commit === "saving");
  const applying = useReplacementText((state) => state.applying);
  const importing = useReplacementFiles((state) => state.phase === "applying");
  return saving || applying || importing;
};

/** Drop the edits of both views and show the saved list; ignored while a commit runs. */
export const discardEditor = (): void => {
  if (isCommitInFlight()) return;
  const editor = useReplacementEditor.getState();
  editor.discard();
  useReplacementText.getState().open(editor.saved);
  editor.setView("list");
};

export const useEditorLifecycle = (): void => {
  useEffect(() => {
    void useReplacementEditor.getState().loadRules();
    const unregister = registerUnsavedChangesGuard({
      isDirty: isEditorDirty,
      isBusy: isCommitInFlight,
      discard: discardEditor,
    });
    const unlisten = listen<{ revision: number }>(
      "settings-changed",
      (event) => {
        void useReplacementEditor
          .getState()
          .followSettingsChange(event.payload.revision);
      },
    );
    return () => {
      unregister();
      void unlisten.then((stop) => stop());
    };
  }, []);
  useTextFollowsSavedRules();
};

/** An untouched text view shows the saved rules again when they change. */
const useTextFollowsSavedRules = (): void => {
  const saved = useReplacementEditor((state) => state.saved);
  useEffect(() => {
    const text = useReplacementText.getState();
    if (!isTextDirty(text) && !text.applying) text.open(saved);
  }, [saved]);
};
