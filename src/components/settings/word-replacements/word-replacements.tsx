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
import { useFocusRescue } from "../../ui/focus-return";
import {
  type EditorView,
  isListDirty,
  useReplacementEditor,
} from "./editor-store";
import { useReplacementFiles } from "./file-store";
import { useReplacementText } from "./text-store";
import { screenId, type ScreenId } from "./rules-model";
import { showsRuleList } from "./editor-model";
import { useCommitInFlight, useEditorLifecycle } from "./use-editor-lifecycle";
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
  const [adding, setAdding] = useState(false);
  const screen = useScreen(view, adding);
  // Stable focus point after Save, Discard, an inline edit or a dialog (native session N5).
  const titleRef = useRef<HTMLHeadingElement>(null);
  const rescueRef = useFocusRescue(titleRef);
  const { gate, dialog } = useDraftGate(titleRef);
  const actions = useEditorActions(gate);
  return (
    <div ref={rescueRef} className="space-y-6" data-testid={`${screen}-screen`}>
      <div data-testid={`${screen}-replacements`}>
        <SettingsGroup
          title={t("settings.replacements.group")}
          titleRef={titleRef}
        >
          <div className="p-4 space-y-3" data-testid={`${screen}-rules`}>
            {/* A refused import file belongs to this section: under its title (N6). */}
            <ImportScreen returnFocusRef={titleRef} />
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

/**
 * Contract screen shown: the list stays while it has rules, an add form was opened or the draft
 * changed (`showsRuleList`), else the empty state.
 * @param view rule view
 * @param adding whether the add form was opened
 * @returns the screen's test-id prefix
 */
const useScreen = (view: EditorView, adding: boolean): ScreenId => {
  const ruleCount = useReplacementEditor((state) => state.draft.length);
  const dirty = useReplacementEditor(isListDirty);
  return screenId(view, showsRuleList(ruleCount, adding, dirty));
};

interface EditorActions {
  importing: boolean;
  exporting: boolean;
  busy: boolean;
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
  const busy = useCommitInFlight();
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
  return { importing, exporting, busy, onView, onImport, onExport };
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
  const load = useReplacementEditor((state) => state.load);
  const hasRules = useReplacementEditor((state) => state.draft.length > 0);
  const recognizedRef = useRef<HTMLInputElement>(null);
  const emptyAddRef = useRef<HTMLButtonElement>(null);
  // The add form's field, or Add rule when the list gave way to the empty state.
  const focusAddForm = () =>
    (recognizedRef.current ?? emptyAddRef.current)?.focus();
  if (load !== "ready") return <LoadState load={load} />;
  if (screen === "replacement-settings-bulk") return <BulkEditor />;
  if (screen === "replacement-settings-empty") {
    return (
      <RulesEmpty addRef={emptyAddRef} onAdd={onAdding} onPaste={onPaste} />
    );
  }
  return (
    <>
      <RuleAddForm
        recognizedRef={recognizedRef}
        focusOnOpen={screen === "replacement-settings" && !hasRules}
        onAdded={onAdding}
      />
      <RuleList onFocusAddForm={focusAddForm} />
    </>
  );
};

/** Saved rules being read, or the failed read with Retry. */
const LoadState: React.FC<{ load: "loading" | "failed" }> = ({ load }) => {
  const { t } = useTranslation();
  if (load === "failed") return <LoadFailed />;
  return (
    <p className="text-sm text-mid-gray">
      {t("settings.replacements.loading")}
    </p>
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
