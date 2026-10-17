// "Prompts" group of the prompt settings (005 AC-001, AC-002, AC-004): every
// prompt with its provider, shortcuts and, when it cannot run, why; the
// default prompt highlighted; New prompt within the 50-prompt limit; a
// failed reload flagged; the focus back on the button that opened the
// editor once it closes.
import React from "react";
import { useTranslation } from "react-i18next";
import { Pencil, Plus } from "lucide-react";
import type { ListedPrompt, PostProcessProvider } from "@/bindings";
import { useOsType } from "../../../hooks/useOsType";
import { useSettings } from "../../../hooks/useSettings";
import { formatKeyCombination } from "../../../lib/utils/keyboard";
import { Alert } from "../../ui/Alert";
import Badge from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui/SettingsGroup";
import {
  MAX_PROMPTS,
  NEW_PROMPT_FOCUS,
  PROMPT_KEYS,
  rowStatusOf,
} from "./prompt-library-model";
import { LibraryReloadNotice } from "./prompt-library-notice";
import { providerLabelOf } from "./prompt-provider-model";
import { useFocusOnMount } from "./use-focus-on-mount";
import type { PromptLibraryState } from "./use-prompt-library";

const TEST_ID = "prompt-settings";

/** Provider line of a row: the override, else the default provider. */
const useProviderText = (prompt: ListedPrompt): string => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const providers: PostProcessProvider[] =
    getSetting("post_process_providers") ?? [];
  const override = prompt.provider_override;
  if (!override) return t(`${PROMPT_KEYS}.list.defaultProvider`);
  return t(`${PROMPT_KEYS}.list.providerModel`, {
    provider: providerLabelOf(providers, override.provider_id),
    model: override.model,
  });
};

/** Shortcut badge, or the invalid/not-ready flag of the row (AC-002, AC-004). */
const RowBadges: React.FC<{ prompt: ListedPrompt }> = ({ prompt }) => {
  const { t } = useTranslation();
  const osType = useOsType();
  const status = rowStatusOf(prompt);
  const shortcuts = prompt.shortcuts.map((s) =>
    formatKeyCombination(s, osType),
  );
  const statusKey = `${PROMPT_KEYS}.list.${status?.kind === "invalid" ? "invalid" : "notReady"}`;
  return (
    <>
      {status && (
        <Badge
          variant={status.kind === "invalid" ? "error" : "warning"}
          testId={`${TEST_ID}-prompt-status`}
        >
          {t(statusKey, { reason: t(status.reasonKey) })}
        </Badge>
      )}
      {(shortcuts.length > 0 || !status) && (
        <Badge variant="secondary" testId={`${TEST_ID}-prompt-shortcut`}>
          {shortcuts.length > 0
            ? shortcuts.join(", ")
            : t(`${PROMPT_KEYS}.list.noShortcut`)}
        </Badge>
      )}
    </>
  );
};

interface PromptRowProps {
  prompt: ListedPrompt;
  isDefault: boolean;
  /** The edit button takes the focus (the editor of this prompt closed). */
  takesFocus: boolean;
  onEdit: (id: string) => void;
}

/** Pencil button that opens the editor of a prompt. */
const EditButton: React.FC<PromptRowProps> = ({
  prompt,
  takesFocus,
  onEdit,
}) => {
  const { t } = useTranslation();
  const focusRef = useFocusOnMount<HTMLButtonElement>(takesFocus);
  const label = t(`${PROMPT_KEYS}.list.edit`, { name: prompt.name });
  return (
    <button
      ref={focusRef}
      type="button"
      className="p-1 rounded-md cursor-pointer hover:bg-logo-primary/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary"
      data-testid={`${TEST_ID}-prompt-edit`}
      aria-label={label}
      title={label}
      onClick={() => onEdit(prompt.id)}
    >
      <Pencil size={16} />
    </button>
  );
};

const PromptRow: React.FC<PromptRowProps> = (props) => {
  const { prompt, isDefault } = props;
  const providerText = useProviderText(prompt);
  return (
    <li
      className={`flex items-center gap-3 px-4 py-2 ${isDefault ? "bg-logo-primary/20" : ""}`}
      data-testid={`${TEST_ID}-prompt`}
      data-instance-id={`prompt-${prompt.id}`}
      data-key={`prompt-${prompt.id}`}
      aria-current={isDefault ? "true" : undefined}
    >
      <div className="flex-1 min-w-0">
        <p
          className="text-sm font-medium truncate"
          data-testid={`${TEST_ID}-prompt-name`}
        >
          {prompt.name}
        </p>
        <p
          className="text-xs text-mid-gray truncate"
          data-testid={`${TEST_ID}-prompt-provider`}
        >
          {providerText}
        </p>
      </div>
      <RowBadges prompt={prompt} />
      <EditButton {...props} />
    </li>
  );
};

/** First load failed: nothing to show but the error and Retry. */
const LoadError: React.FC<{ state: PromptLibraryState }> = ({ state }) => {
  const { t } = useTranslation();
  const retry = (
    <Button variant="secondary" size="sm" onClick={state.reload}>
      {t(`${PROMPT_KEYS}.list.retry`)}
    </Button>
  );
  return (
    <Alert
      variant="error"
      role="alert"
      testId={`${TEST_ID}-load-error`}
      action={retry}
    >
      {t(`${PROMPT_KEYS}.list.loadFailed`)}
    </Alert>
  );
};

/** Loading, load failure or empty library, in place of the list. */
const ListPlaceholder: React.FC<{ state: PromptLibraryState }> = ({
  state,
}) => {
  const { t } = useTranslation();
  if (state.loadState === "error" && !state.library) {
    return <LoadError state={state} />;
  }
  const isLoading = !state.library;
  return (
    <p
      role="status"
      className="p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20 text-sm text-mid-gray"
      data-testid={`${TEST_ID}-${isLoading ? "loading" : "empty"}`}
    >
      {t(`${PROMPT_KEYS}.list.${isLoading ? "loading" : "empty"}`)}
    </p>
  );
};

interface PromptListGroupProps {
  state: PromptLibraryState;
  /** Prompt id or `NEW_PROMPT_FOCUS` whose button takes the focus back. */
  focusId: string | null;
  onEdit: (id: string) => void;
  onNew: () => void;
}

interface ListFooterProps {
  count: number;
  takesFocus: boolean;
  onNew: () => void;
}

/** New prompt and the library count; New is unavailable at the limit. */
const ListFooter: React.FC<ListFooterProps> = ({
  count,
  takesFocus,
  onNew,
}) => {
  const { t } = useTranslation();
  const focusRef = useFocusOnMount<HTMLDivElement>(takesFocus);
  const isFull = count >= MAX_PROMPTS;
  return (
    <div ref={focusRef} className="flex flex-wrap items-center gap-3">
      <Button
        variant="secondary"
        size="md"
        className="inline-flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-logo-primary"
        onClick={onNew}
        disabled={isFull}
        data-testid={`${TEST_ID}-new`}
      >
        <Plus size={16} aria-hidden="true" />
        {t(`${PROMPT_KEYS}.list.new`)}
      </Button>
      <span className="text-sm text-mid-gray" data-testid={`${TEST_ID}-count`}>
        {isFull
          ? t(`${PROMPT_KEYS}.list.full`, { limit: MAX_PROMPTS })
          : t(`${PROMPT_KEYS}.list.count`, { count, limit: MAX_PROMPTS })}
      </span>
    </div>
  );
};

/** The rows of a non-empty library. */
const PromptRows: React.FC<PromptListGroupProps> = ({
  state,
  focusId,
  onEdit,
}) => {
  const defaultId = state.library?.default_prompt_id ?? null;
  return (
    <ul
      className="rounded-md border border-mid-gray/20 divide-y divide-mid-gray/20 overflow-hidden"
      data-testid={`${TEST_ID}-list`}
    >
      {(state.library?.prompts ?? []).map((prompt) => (
        <PromptRow
          key={prompt.id}
          prompt={prompt}
          isDefault={prompt.id === defaultId}
          takesFocus={prompt.id === focusId}
          onEdit={onEdit}
        />
      ))}
    </ul>
  );
};

/** The library with its rows, New prompt and count. */
export const PromptListGroup: React.FC<PromptListGroupProps> = (props) => {
  const { state, focusId, onNew } = props;
  const { t } = useTranslation();
  const count = state.library?.prompts.length ?? 0;
  return (
    <div data-testid={`${TEST_ID}-library`}>
      <SettingsGroup title={t(`${PROMPT_KEYS}.list.title`)}>
        <div className="px-4 py-3 space-y-3">
          <LibraryReloadNotice testId={TEST_ID} state={state} />
          {count > 0 ? (
            <PromptRows {...props} />
          ) : (
            <ListPlaceholder state={state} />
          )}
          {state.library && (
            <ListFooter
              count={count}
              takesFocus={focusId === NEW_PROMPT_FOCUS}
              onNew={onNew}
            />
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};
