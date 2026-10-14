/**
 * Vocabulary editor (feature 003), which replaces the former "Custom Words" editor on the same
 * `custom_words` list: saved terms with their legacy flags, Add and per-occurrence removal, a
 * paste box with preview and Apply, and the status for the selected model. Every commit goes
 * through the vocabulary commands with the revision the editor shows.
 */
import React, { useCallback, useId } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useStore } from "zustand";
import { useVocabulary } from "../../hooks/use-vocabulary";
import { useModelStore } from "../../stores/modelStore";
import { useSettingsStore } from "../../stores/settingsStore";
import type { VocabularyStore } from "../../stores/vocabulary-store";
import { getTranslatedModelName } from "../../lib/utils/modelTranslation";
import { Alert } from "../ui/Alert";
import { Button } from "../ui/Button";
import { SettingsGroup } from "../ui/SettingsGroup";
import {
  canRetry,
  commitErrorKey,
  isTermRefusal,
  listScreen,
  overBudgetIndices,
  VOCABULARY_SCREENS,
  type VocabularyScreen,
} from "./vocabulary/vocabulary-model";
import { VocabularyStatusView } from "./vocabulary/vocabulary-status";
import {
  VocabularyAddRow,
  VocabularyCount,
  VocabularyTerms,
  type AddRowProps,
  type TermListProps,
} from "./vocabulary/vocabulary-terms";
import {
  VocabularyApplyBar,
  VocabularyPaste,
} from "./vocabulary/vocabulary-paste";

/** Display name of the status model, or of the selected one before any status answers. */
const useModelName = (store: VocabularyStore): string => {
  const { t } = useTranslation();
  const status = useStore(store, (state) => state.status);
  const selected = useSettingsStore((state) => state.settings?.selected_model);
  const modelId = status.kind === "ready" ? status.report.model_id : selected;
  const info = useModelStore((state) =>
    state.models.find((model) => model.id === modelId),
  );
  if (!modelId) return t("settings.vocabulary.status.selectedModel");
  return info ? getTranslatedModelName(info, t) : modelId;
};

interface ViewProps {
  store: VocabularyStore;
  screen: VocabularyScreen;
}

/** The failed commit, if any, with Retry when retrying can succeed. */
const FailureView: React.FC<ViewProps & { errorId: string }> = (props) => {
  const { store, screen, errorId } = props;
  const { t } = useTranslation();
  const failure = useStore(store, (state) => state.failure);
  const retry = useStore(store, (state) => state.retry);
  if (!failure) return null;
  return (
    <div
      className="flex flex-col items-start gap-2"
      data-testid={`${screen}-error`}
      role="alert"
    >
      <Alert variant="error">
        <span id={errorId} className="block">
          {t(commitErrorKey(failure.error))}
        </span>
        <span className="block">{t("settings.vocabulary.errors.saved")}</span>
      </Alert>
      {canRetry(failure.error) && (
        <Button variant="secondary" size="sm" onClick={() => void retry()}>
          {t("settings.vocabulary.retry")}
        </Button>
      )}
    </div>
  );
};

const Header: React.FC<{ screen: VocabularyScreen; paste: boolean }> = ({
  screen,
  paste,
}) => {
  const { t } = useTranslation();
  const root = paste ? "settings.vocabulary.pasteBox" : "settings.vocabulary";
  return (
    <div className="flex flex-col gap-0.5 text-sm">
      <h3 className="font-medium" data-testid={`${screen}-terms-title`}>
        {t(`${root}.title`)}
      </h3>
      <p>{t(`${root}.description`)}</p>
    </div>
  );
};

/** Props of the saved-term chips, read from the store. */
const useTermsProps = (
  store: VocabularyStore,
  screen: VocabularyScreen,
  focusInput: () => void,
): TermListProps => {
  const list = useStore(store, (state) => state.list);
  const status = useStore(store, (state) => state.status);
  const pending = useStore(store, (state) => state.pending);
  const remove = useStore(store, (state) => state.remove);
  return {
    terms: list?.terms ?? [],
    screen,
    overBudget: overBudgetIndices(
      status.kind === "ready" ? status.report : null,
    ),
    busy: pending !== null || list === null,
    onRemove: (occurrence) => remove(occurrence.index, occurrence.term),
    focusInput,
  };
};

/** Props of the Add row, read from the store. */
const useAddRowProps = (
  store: VocabularyStore,
  screen: VocabularyScreen,
  ids: { inputId: string; errorId: string },
): AddRowProps => {
  const state = useStore(store);
  const failure = state.failure;
  const termRefused =
    failure?.operation.kind === "add" && isTermRefusal(failure.error);
  return {
    screen,
    draft: state.draft,
    busy: state.pending !== null || state.list === null,
    refusal: termRefused ? failure.error : null,
    errorId: failure ? ids.errorId : undefined,
    inputId: ids.inputId,
    onDraft: state.setDraft,
    onAdd: () => void state.add(),
    onPaste: state.openPaste,
  };
};

/** List view: status, terms, Add row, error and count. */
const ListView: React.FC<ViewProps> = ({ store, screen }) => {
  const inputId = useId();
  const errorId = useId();
  const state = useStore(store);
  const model = useModelName(store);
  const focusInput = useCallback(
    () => document.getElementById(inputId)?.focus(),
    [inputId],
  );
  const termsProps = useTermsProps(store, screen, focusInput);
  const addRowProps = useAddRowProps(store, screen, { inputId, errorId });
  const total = termsProps.terms.length;
  return (
    <>
      <VocabularyStatusView
        status={state.status}
        screen={screen}
        model={model}
        onRetry={() => void state.loadStatus()}
      />
      <VocabularyTerms {...termsProps} />
      <VocabularyAddRow {...addRowProps} />
      <FailureView store={store} screen={screen} errorId={errorId} />
      {total > 0 && <VocabularyCount screen={screen} total={total} />}
    </>
  );
};

/** Paste view body inside the card: editor, preview and an Apply failure. */
const PasteView: React.FC<ViewProps> = ({ store, screen }) => {
  const errorId = useId();
  const state = useStore(store);
  const fresh = state.previewText === state.pasteText;
  return (
    <>
      <VocabularyPaste
        text={state.pasteText}
        preview={fresh ? state.preview : null}
        busy={state.pending?.kind === "apply"}
        onText={state.setPasteText}
      />
      <FailureView store={store} screen={screen} errorId={errorId} />
    </>
  );
};

/** Apply bar under the paste card; a successful Apply is confirmed with a toast. */
const ApplyBar: React.FC<{ store: VocabularyStore }> = ({ store }) => {
  const { t } = useTranslation();
  const state = useStore(store);
  const apply = async () => {
    if (await state.apply()) {
      toast.success(t("settings.vocabulary.pasteBox.applied"));
    }
  };
  return (
    <VocabularyApplyBar
      text={state.pasteText}
      preview={state.preview}
      fresh={state.previewText === state.pasteText}
      applying={state.pending?.kind === "apply"}
      onCancel={state.closePaste}
      onApply={() => void apply()}
    />
  );
};

/**
 * Settings group "Vocabulary": the list view, or the paste view with its Apply bar.
 */
export const CustomWords: React.FC = () => {
  const { t } = useTranslation();
  const store = useVocabulary();
  const state = useStore(store);
  const report = state.status.kind === "ready" ? state.status.report : null;
  const screen = state.pasteOpen
    ? VOCABULARY_SCREENS.paste
    : listScreen(state.list?.terms.length ?? 0, report?.status ?? null);
  const View = state.pasteOpen ? PasteView : ListView;
  return (
    <div className="space-y-6" data-testid={`${screen}-screen`}>
      <div data-testid={`${screen}-vocabulary`} data-state={state.phase}>
        <SettingsGroup title={t("settings.vocabulary.group")}>
          <div
            className="flex flex-col gap-3 px-4 py-2"
            data-testid={`${screen}-terms`}
          >
            <Header screen={screen} paste={state.pasteOpen} />
            <View store={store} screen={screen} />
          </div>
        </SettingsGroup>
      </div>
      {state.pasteOpen && <ApplyBar store={store} />}
    </div>
  );
};
