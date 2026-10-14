// Microphone priority editor of the audio settings (feature 001), replacing the single microphone
// selector. The editor state is shared by three places of the page: the conflict banner above the
// Sound group, the rows inside it (priority, in use now, closed-lid) and the Unsaved changes bar
// below it.
import React, { createContext, useContext } from "react";
import { ClamshellRow } from "./clamshell-row";
import { ConflictAlert, DraftBar } from "./draft-feedback";
import { orderError } from "./draft-state";
import { EffectiveInputRow } from "./effective-input";
import { PrioritySection } from "./priority-editor";
import { usePriorityEditor, type PriorityEditor } from "./use-priority-editor";

const EditorContext = createContext<PriorityEditor | null>(null);

/** Draft bar bound to the editor. */
const EditorDraftBar: React.FC<{ editor: PriorityEditor }> = ({ editor }) => (
  <DraftBar
    visible={editor.dirty}
    phase={editor.draft.savePhase}
    error={editor.draft.saveError ?? orderError(editor.order)}
    saveEnabled={editor.saveEnabled}
    onDiscard={editor.discard}
    onSave={() => void editor.save()}
  />
);

/**
 * Screen of the microphone priority editor around the Sound group.
 * Side effects: reads the microphone status every 5 s while mounted.
 * @param props.children the Sound settings group, which renders MicrophonePriorityRows
 */
export const MicrophonePrioritySettings: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const editor = usePriorityEditor();
  return (
    <EditorContext.Provider value={editor}>
      <div className="space-y-6" data-testid="microphone-settings-screen">
        <ConflictAlert
          visible={editor.draft.conflict}
          onReload={() => void editor.reload()}
        />
        <div data-testid="microphone-settings-sound">{children}</div>
        <EditorDraftBar editor={editor} />
      </div>
    </EditorContext.Provider>
  );
};

/** Rows of the Sound group: priority list, In use now and the closed-lid microphone. */
export const MicrophonePriorityRows: React.FC = () => {
  const editor = useContext(EditorContext);
  if (editor === null) return null;
  return (
    <>
      <PrioritySection editor={editor} />
      <EffectiveInputRow view={editor.status} />
      <ClamshellRow />
    </>
  );
};
