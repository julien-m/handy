// Prompt editor of the AI prompt settings (005 Steps 6, flow
// `custom-ai-prompts`): the form (name, text, provider override, shortcuts),
// the test panel and the action bar. Editing a saved prompt is the
// contract's `prompt-settings-editor` screen; a new prompt is its
// `prompt-settings-invalid` screen (title "New prompt").
import React, { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { PromptLibraryView } from "@/bindings";
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
import { PromptProviderFields } from "./prompt-provider-fields";
import { type ReadinessFix, overrideOf } from "./prompt-provider-model";
import { PromptTestPanel } from "./prompt-test-panel";
import { type PromptEditorModel, usePromptEditor } from "./use-prompt-editor";
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

/** Shortcuts of a saved prompt, through the shared 004 list. */
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
    </div>
  );
};

interface FormGroupProps extends PartProps {
  target: EditorTarget;
  onFix: (fix: ReadinessFix) => void;
}

/** The "Edit prompt" / "New prompt" group. */
const EditorFormGroup: React.FC<FormGroupProps> = ({
  testId,
  model,
  target,
  onFix,
}) => {
  const { t } = useTranslation();
  const title = t(
    `${KEY}.${target.mode === "edit" ? "editTitle" : "newTitle"}`,
  );
  return (
    <div data-testid={`${testId}-editor`}>
      <SettingsGroup title={title}>
        <div className="p-4 space-y-4" data-testid={`${testId}-form`}>
          <ExternalChangeNotice testId={testId} model={model} />
          <PromptNameField testId={testId} model={model} />
          <PromptTextField testId={testId} model={model} />
          <PromptProviderFields testId={testId} model={model} onFix={onFix} />
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
  library: PromptLibraryView | null;
  /** A new prompt now exists: edit it from now on. */
  onCreated: (id: string) => void;
  /** Back to the prompt list. */
  onClose: () => void;
  /** Back to the list, at the provider settings where `fix` is made. */
  onFix: (fix: ReadinessFix) => void;
}

// @spec AC-001: Prompt create, edit and delete with confirmation — .specs/features/005-custom-ai-prompts/spec.md#ac-001
/** The editor screen. */
export const PromptEditor: React.FC<PromptEditorProps> = (props) => {
  const { target, library, onCreated, onClose, onFix } = props;
  const testId = editorTestIdOf(target);
  const model = usePromptEditor(target, library, onCreated);
  const test = usePromptTest((sample) => ({
    text: model.draft.text,
    provider_override: overrideOf(model.draft.provider),
    sample,
  }));
  const saved =
    target.mode === "edit"
      ? library?.prompts.find((prompt) => prompt.id === target.id)
      : undefined;
  const deletion = usePromptDeletion(saved, onClose);
  const rootRef = useRef<HTMLDivElement>(null);
  // The page keeps the list's scroll position: show the editor from its top.
  useEffect(() => {
    rootRef.current?.scrollIntoView({ block: "start" });
  }, []);
  return (
    <div
      ref={rootRef}
      className="max-w-3xl w-full mx-auto space-y-6"
      data-testid={`${testId}-screen`}
    >
      <EditorFormGroup
        testId={testId}
        model={model}
        target={target}
        onFix={onFix}
      />
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
          isDefault={library?.default_prompt_id === saved.id}
          deletion={deletion}
        />
      )}
    </div>
  );
};
