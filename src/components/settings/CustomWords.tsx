/**
 * Vocabulary editor (feature 003), which replaces the former "Custom Words" editor on the same
 * `custom_words` list: saved terms with their legacy flags, Add and per-occurrence removal, a
 * paste box with preview and Apply, and the status for the selected model. Every commit goes
 * through the vocabulary commands with the revision the editor shows.
 */
import React, { useCallback, useEffect, useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useStore } from "zustand";
import { useVocabulary } from "../../hooks/use-vocabulary";
import { useModelStore } from "../../stores/modelStore";
import { useSettingsStore } from "../../stores/settingsStore";
import type {
  VocabularyState,
  VocabularyStore,
} from "../../stores/vocabulary-store";
import { getTranslatedModelName } from "../../lib/utils/modelTranslation";
import { SettingsGroup } from "../ui/SettingsGroup";
import { useFocusRescue } from "../ui/focus-return";
import {
  isTermRefusal,
  listScreen,
  overBudgetIndices,
  VOCABULARY_SCREENS,
  type VocabularyScreen,
} from "./vocabulary/vocabulary-model";
import {
  CommitFailure,
  ListLoading,
  ListReadFailure,
} from "./vocabulary/vocabulary-notices";
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
import { retryApply, retryThenFocus } from "./vocabulary/vocabulary-retry";

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
const FailureView: React.FC<
  ViewProps & { errorId: string; onRetry: () => Promise<void> }
> = (props) => {
  const failure = useStore(props.store, (state) => state.failure);
  if (!failure) return null;
  return (
    <CommitFailure
      screen={props.screen}
      error={failure.error}
      errorId={props.errorId}
      onRetry={() => void props.onRetry()}
    />
  );
};

/**
 * Retry of a failed Apply with its messages: why nothing was sent, or the same confirmation as
 * Apply.
 * @param store editor store
 * @returns the Retry action
 */
const useApplyRetry = (store: VocabularyStore): (() => Promise<void>) => {
  const { t } = useTranslation();
  return () =>
    retryApply(store, {
      blocked: ({ note, tone }) =>
        (tone === "info" ? toast.info : toast.warning)(
          t(note.key, note.values),
        ),
      applied: () => toast.success(t("settings.vocabulary.pasteBox.applied")),
    });
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
  ids: { inputId: string; errorId: string; pasteId: string },
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
    pasteId: ids.pasteId,
    onDraft: state.setDraft,
    onAdd: () => void state.add(),
    onPaste: state.openPaste,
  };
};

/** Saved terms once read; a loading line before, an error with Retry when the read failed. */
const SavedTerms: React.FC<ViewProps & { termsProps: TermListProps }> = (
  props,
) => {
  const { store, screen, termsProps } = props;
  const read = useStore(store, (state) => state.list !== null);
  const failed = useStore(store, (state) => state.listFailed);
  const loadList = useStore(store, (state) => state.loadList);
  // The error and its Retry leave once the list is read: hand the focus to the term input.
  const retry = async () => {
    await loadList();
    if (!store.getState().listFailed) termsProps.focusInput();
  };
  return (
    <>
      {failed && (
        <ListReadFailure screen={screen} onRetry={() => void retry()} />
      )}
      {read && <VocabularyTerms {...termsProps} />}
      {!read && !failed && <ListLoading screen={screen} />}
    </>
  );
};

/** Status of the vocabulary for the selected model, with its Retry. */
const StatusView: React.FC<ViewProps> = ({ store, screen }) => {
  const status = useStore(store, (state) => state.status);
  const loadStatus = useStore(store, (state) => state.loadStatus);
  const model = useModelName(store);
  return (
    <VocabularyStatusView
      status={status}
      screen={screen}
      model={model}
      onRetry={() => void loadStatus()}
    />
  );
};

/** List view: status, terms, Add row, error and count. */
const ListView: React.FC<ViewProps & { pasteId: string }> = (props) => {
  const { store, screen, pasteId } = props;
  const inputId = useId();
  const errorId = useId();
  const focusInput = useCallback(
    () => document.getElementById(inputId)?.focus(),
    [inputId],
  );
  const termsProps = useTermsProps(store, screen, focusInput);
  const ids = { inputId, errorId, pasteId };
  const addRowProps = useAddRowProps(store, screen, ids);
  const total = termsProps.terms.length;
  return (
    <>
      <StatusView store={store} screen={screen} />
      <SavedTerms store={store} screen={screen} termsProps={termsProps} />
      <VocabularyAddRow {...addRowProps} />
      <FailureView
        store={store}
        screen={screen}
        errorId={errorId}
        onRetry={retryThenFocus(store, focusInput)}
      />
      {total > 0 && <VocabularyCount screen={screen} total={total} />}
    </>
  );
};

/** Paste view body inside the card: editor, preview and an Apply failure. */
const PasteView: React.FC<ViewProps> = ({ store, screen }) => {
  const errorId = useId();
  const state = useStore(store);
  const retryFailedApply = useApplyRetry(store);
  const fresh = state.previewText === state.pasteText;
  return (
    <>
      <VocabularyPaste
        text={state.pasteText}
        preview={fresh ? state.preview : null}
        busy={state.pending?.kind === "apply"}
        onText={state.setPasteText}
      />
      <FailureView
        store={store}
        screen={screen}
        errorId={errorId}
        onRetry={retryFailedApply}
      />
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

/** When the paste box closes (Cancel or Apply), give the focus back to "Paste a list…". */
const usePasteReturnFocus = (pasteOpen: boolean, pasteId: string) => {
  const wasOpen = useRef(pasteOpen);
  useEffect(() => {
    if (wasOpen.current && !pasteOpen) {
      document.getElementById(pasteId)?.focus();
    }
    wasOpen.current = pasteOpen;
  }, [pasteOpen, pasteId]);
};

/**
 * Contract screen shown: the paste screen while the box is open, else the list screen that fits
 * the saved terms and the model status.
 * @param state editor state
 * @returns the screen's test-id prefix
 */
const vocabularyScreen = (
  state: Pick<VocabularyState, "pasteOpen" | "list" | "status">,
): VocabularyScreen => {
  if (state.pasteOpen) return VOCABULARY_SCREENS.paste;
  const report = state.status.kind === "ready" ? state.status.report : null;
  return listScreen(state.list?.terms.length ?? null, report?.status ?? null);
};

/**
 * Settings group "Vocabulary": the list view, or the paste view with its Apply bar.
 */
export const CustomWords: React.FC = () => {
  const { t } = useTranslation();
  const store = useVocabulary();
  const state = useStore(store);
  const pasteId = useId();
  usePasteReturnFocus(state.pasteOpen, pasteId);
  const screen = vocabularyScreen(state);
  // Stable focus point when the focused control leaves (native session N5).
  const titleRef = useRef<HTMLHeadingElement>(null);
  const rescueRef = useFocusRescue(titleRef);
  return (
    <div ref={rescueRef} className="space-y-6" data-testid={`${screen}-screen`}>
      <div data-testid={`${screen}-vocabulary`} data-state={state.phase}>
        <SettingsGroup
          title={t("settings.vocabulary.group")}
          titleRef={titleRef}
        >
          <div
            className="flex flex-col gap-3 px-4 py-2"
            data-testid={`${screen}-terms`}
          >
            <Header screen={screen} paste={state.pasteOpen} />
            {state.pasteOpen ? (
              <PasteView store={store} screen={screen} />
            ) : (
              <ListView store={store} screen={screen} pasteId={pasteId} />
            )}
          </div>
        </SettingsGroup>
      </div>
      {state.pasteOpen && <ApplyBar store={store} />}
    </div>
  );
};
