// Prompt editor of the AI prompt settings (005 Step 6, flow
// `custom-ai-prompts`): the form (name, text, provider override, shortcuts),
// the test panel and the action bar. The name field takes the focus when
// the editor opens. Editing a saved prompt is the
// contract's `prompt-settings-editor` screen; a new prompt is its
// `prompt-settings-invalid` screen (title "New prompt").
import React, { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { ListedPrompt, PromptLibraryView } from "@/bindings";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { ShortcutListEditor } from "../shortcut-list";
import {
  MAX_SHORTCUTS_PER_PROMPT,
  promptTargetOf,
} from "../shortcut-list-model";
import { type EditorTarget, editorTestIdOf } from "./prompt-draft-model";
import { PromptDeleteDialog, usePromptDeletion } from "./prompt-delete-dialog";
import { PromptEditorActions } from "./prompt-editor-actions";
import { PromptNameField, PromptTextField } from "./prompt-editor-fields";
import { PROMPT_KEYS } from "./prompt-library-model";
import { LibraryReloadNotice } from "./prompt-library-notice";
import { PromptProviderFields } from "./prompt-provider-fields";
import { overrideOf } from "./prompt-provider-model";
import { PromptTestPanel } from "./prompt-test-panel";
import { type PromptEditorModel, usePromptEditor } from "./use-prompt-editor";
import type { PromptLibraryState } from "./use-prompt-library";
import { usePromptTest } from "./use-prompt-test";

const KEY = `${PROMPT_KEYS}.editor`;

interface PartProps {
  testId: string;
  model: PromptEditorModel;
}

/** Notice of a change made elsewhere to the edited prompt. */
const ExternalChangeNotice: React.FC<PartProps> = ({ testId, model }) => {
  const { t } = useTranslation();
  if (model.external === "deleted") {
    return (
      <Alert
        variant="error"
        role="alert"
        testId={`${testId}-deleted-elsewhere`}
      >
        {t(`${KEY}.deletedElsewhere`)}
      </Alert>
    );
  }
  if (model.external !== "changed") return null;
  return (
    <Alert
      variant="warning"
      role="status"
      testId={`${testId}-changed-elsewhere`}
      action={
        <Button variant="warning" size="sm" onClick={model.reloadSaved}>
          {t(`${KEY}.reload`)}
        </Button>
      }
    >
      {t(`${KEY}.changedElsewhere`)}
    </Alert>
  );
};

/**
 * Shortcuts of a saved prompt, through the shared 004 list. They are saved
 * as soon as they change, not by Save, and the hint says so.
 */
const PromptShortcuts: React.FC<{ testId: string; promptId: string }> = ({
  testId,
  promptId,
}) => {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-2">
      <span
        className="text-sm font-semibold"
        data-testid={`${testId}-form-shortcuts-label`}
      >
        {t(`${KEY}.shortcuts`, { limit: MAX_SHORTCUTS_PER_PROMPT })}
      </span>
      <ShortcutListEditor
        actionId={promptTargetOf(promptId)}
        layout="row"
        testIds={{
          list: `${testId}-form-shortcuts`,
          row: () => `${testId}-form-shortcut`,
          add: `${testId}-form-add-shortcut`,
        }}
      />
      <p
        className="text-xs text-mid-gray/70"
        data-testid={`${testId}-form-shortcuts-hint`}
      >
        {t(`${KEY}.shortcutsSavedImmediately`)}
      </p>
    </div>
  );
};

interface FormGroupProps extends PartProps {
  target: EditorTarget;
  libraryState: PromptLibraryState;
}

/** The "Edit prompt" / "New prompt" group. */
const EditorFormGroup: React.FC<FormGroupProps> = (props) => {
  const { testId, model, target, libraryState } = props;
  const { t } = useTranslation();
  const title = t(
    `${KEY}.${target.mode === "edit" ? "editTitle" : "newTitle"}`,
  );
  return (
    <div data-testid={`${testId}-editor`}>
      <SettingsGroup title={title}>
        <div className="p-4 space-y-4" data-testid={`${testId}-form`}>
          <LibraryReloadNotice testId={testId} state={libraryState} />
          <ExternalChangeNotice testId={testId} model={model} />
          <PromptNameField testId={testId} model={model} focusOnOpen />
          <PromptTextField testId={testId} model={model} />
          <PromptProviderFields testId={testId} model={model} />
          {target.mode === "edit" && (
            <PromptShortcuts testId={testId} promptId={target.id} />
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};

interface PromptEditorProps {
  target: EditorTarget;
  libraryState: PromptLibraryState;
  /** A new prompt now exists: edit it from now on. */
  onCreated: (id: string) => void;
  /** Back to the prompt list. */
  onClose: () => void;
}

/** Scrolls the editor into view once, from its top. */
const useScrollToTop = () => {
  const rootRef = useRef<HTMLDivElement>(null);
  // The page keeps the list's scroll position: show the editor from its top.
  useEffect(() => {
    rootRef.current?.scrollIntoView({ block: "start" });
  }, []);
  return rootRef;
};

/** The edited prompt as saved in the library, if any. */
const savedPromptOf = (
  target: EditorTarget,
  library: PromptLibraryView | null,
): ListedPrompt | undefined =>
  target.mode === "edit"
    ? library?.prompts.find((prompt) => prompt.id === target.id)
    : undefined;

/** The editor's state: draft, test panel and deletion of a saved prompt. */
const useEditorParts = (props: PromptEditorProps) => {
  const { target, libraryState, onCreated, onClose } = props;
  const model = usePromptEditor(target, libraryState, onCreated);
  const test = usePromptTest({
    text: model.draft.text,
    provider_override: overrideOf(model.draft.provider),
  });
  const saved = savedPromptOf(target, libraryState.library);
  const deletion = usePromptDeletion(saved, onClose);
  const isDefault = libraryState.library?.default_prompt_id === saved?.id;
  return { model, test, saved, deletion, isDefault };
};

// @spec AC-001: Prompt create, edit and delete with confirmation — .specs/features/005-custom-ai-prompts/spec.md#ac-001
/** The editor screen. */
export const PromptEditor: React.FC<PromptEditorProps> = (props) => {
  const { target, libraryState, onClose } = props;
  const testId = editorTestIdOf(target);
  const { model, test, saved, deletion, isDefault } = useEditorParts(props);
  const rootRef = useScrollToTop();
  const formProps = { testId, model, target, libraryState };
  return (
    <div
      ref={rootRef}
      className="max-w-3xl w-full mx-auto space-y-6"
      data-testid={`${testId}-screen`}
    >
      <EditorFormGroup {...formProps} />
      <PromptTestPanel testId={testId} test={test} />
      <PromptEditorActions
        testId={testId}
        model={model}
        deletion={saved ? deletion : null}
        onCancel={onClose}
      />
      {saved && (
        <PromptDeleteDialog
          testId={testId}
          prompt={saved}
          isDefault={isDefault}
          deletion={deletion}
        />
      )}
    </div>
  );
};
