/**
 * Replacement rules section of the settings (feature 002, flow `word-replacements`): the list
 * with its add form and draft, or the text view with its preview, plus file import and export.
 * The root test id follows the contract screen shown: `replacement-settings`, `-empty`, `-bulk`
 * and, for a file import, `-import`.
 */
import React, { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { type EditorView, useReplacementEditor } from "./editor-store";
import { useReplacementFiles } from "./file-store";
import { useReplacementText } from "./text-store";
import { screenId, type ScreenId } from "./rules-model";
import { useEditorLifecycle } from "./use-editor-lifecycle";
import { useDraftGate } from "./draft-gate";
import { RulesHeader } from "./rules-header";
import { RuleAddForm } from "./rule-add-form";
import { RuleList } from "./rule-list";
import { RulesEmpty } from "./rules-empty";
import { DraftBar } from "./draft-bar";
import { BulkEditor } from "./bulk-editor";
import { ApplyBar } from "./apply-bar";
import { ImportScreen } from "./import-dialog";

export const WordReplacements: React.FC = () => {
  const { t } = useTranslation();
  useEditorLifecycle();
  const view = useReplacementEditor((state) => state.view);
  const ruleCount = useReplacementEditor((state) => state.draft.length);
  const [adding, setAdding] = useState(false);
  const screen = screenId(view, ruleCount > 0 || adding);
  const { gate, dialog } = useDraftGate();
  const actions = useEditorActions(gate);
  return (
    <div className="space-y-6" data-testid={`${screen}-screen`}>
      <ImportScreen />
      <div data-testid={`${screen}-replacements`}>
        <SettingsGroup title={t("settings.replacements.group")}>
          <div className="p-4 space-y-3" data-testid={`${screen}-rules`}>
            <RulesHeader screen={screen} view={view} {...actions} />
            <EditorBody
              screen={screen}
              onAdding={() => setAdding(true)}
              onPaste={() => actions.onView("text")}
            />
          </div>
        </SettingsGroup>
      </div>
      {view === "list" ? <DraftBar /> : <ApplyBar />}
      {dialog}
    </div>
  );
};

interface EditorActions {
  importing: boolean;
  exporting: boolean;
  onView: (view: EditorView) => void;
  onImport: () => void;
  onExport: () => void;
}

/** View switch, import and export, each behind the unsaved-edit gate where it matters. */
const useEditorActions = (
  gate: (action: () => void) => void,
): EditorActions => {
  const { t } = useTranslation();
  const importing = useReplacementFiles((state) => state.phase !== "idle");
  const exporting = useReplacementFiles((state) => state.exporting);
  const onView = (target: EditorView) => {
    if (useReplacementEditor.getState().view === target) return;
    gate(() => {
      const editor = useReplacementEditor.getState();
      if (target === "text") useReplacementText.getState().open(editor.saved);
      editor.setView(target);
    });
  };
  const onImport = () =>
    gate(() => void useReplacementFiles.getState().pickImport());
  // Export writes the saved rules; unsaved edits are not part of it, so it needs no gate.
  const onExport = () => void exportWithFeedback(t);
  return { importing, exporting, onView, onImport, onExport };
};

/** Export the saved rules and confirm the outcome with a toast. */
const exportWithFeedback = async (
  t: (key: string, options?: Record<string, unknown>) => string,
): Promise<void> => {
  const outcome = await useReplacementFiles.getState().exportRules();
  if (outcome.status === "cancelled") return;
  if (outcome.status === "failed") {
    toast.error(t(outcome.key, { name: outcome.fileName }));
    return;
  }
  const key = outcome.durable
    ? "settings.replacements.export.done"
    : "settings.replacements.export.notSynced";
  toast.success(t(key, { name: outcome.fileName }));
};

interface EditorBodyProps {
  screen: ScreenId;
  onAdding: () => void;
  onPaste: () => void;
}

/** Loading or failed read, the text view, the empty state, or the list with its add form. */
const EditorBody: React.FC<EditorBodyProps> = ({
  screen,
  onAdding,
  onPaste,
}) => {
  const { t } = useTranslation();
  const load = useReplacementEditor((state) => state.load);
  const hasRules = useReplacementEditor((state) => state.draft.length > 0);
  const recognizedRef = useRef<HTMLInputElement>(null);
  if (load === "loading") {
    return (
      <p className="text-sm text-mid-gray">
        {t("settings.replacements.loading")}
      </p>
    );
  }
  if (load === "failed") return <LoadFailed />;
  if (screen === "replacement-settings-bulk") return <BulkEditor />;
  if (screen === "replacement-settings-empty") {
    return <RulesEmpty onAdd={onAdding} onPaste={onPaste} />;
  }
  return (
    <>
      <RuleAddForm
        recognizedRef={recognizedRef}
        focusOnOpen={screen === "replacement-settings" && !hasRules}
        onAdded={onAdding}
      />
      <RuleList recognizedRef={recognizedRef} />
    </>
  );
};

const LoadFailed: React.FC = () => {
  const { t } = useTranslation();
  return (
    <div className="flex items-center justify-between gap-2" role="alert">
      <p className="text-sm text-red-400">
        {t("settings.replacements.loadFailed")}
      </p>
      <Button
        variant="secondary"
        size="sm"
        onClick={() => void useReplacementEditor.getState().loadRules()}
      >
        {t("settings.replacements.retry")}
      </Button>
    </div>
  );
};
