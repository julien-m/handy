/**
 * Quick Add window (018): two tabs, Replacement (heard → written, 002) and Vocabulary (one term,
 * 003). Enter submits, Escape closes; after a saved addition the window confirms it, clears the
 * fields and stays open for another one. Refusals show the 002/003 reasons and keep the text.
 */
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { commands } from "@/bindings";
import { Alert } from "@/components/ui/Alert";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import {
  EMPTY_VALUES,
  QUICK_ADD_KIND_EVENT,
  QUICK_ADD_SCREEN,
  canSubmit,
  quickAddFailure,
  type QuickAddFailure,
  type QuickAddKind,
  type QuickAddValues,
} from "./quick-add-model";

/** What the window reports under the fields. */
type Notice =
  | { kind: "added"; text: string }
  | { kind: "failed"; failure: QuickAddFailure };

const KINDS: ReadonlyArray<QuickAddKind> = ["replacement", "vocabulary"];

// @spec AC-007: Confirmation, cleared fields, Escape closes — .specs/features/018-quick-add-from-tray/spec.md#ac-007
export const QuickAddWindow: React.FC<{ initialKind: QuickAddKind }> = ({
  initialKind,
}) => {
  const { t } = useTranslation();
  const [kind, setKind] = useState<QuickAddKind>(initialKind);
  const [values, setValues] = useState<QuickAddValues>(EMPTY_VALUES);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<Notice | null>(null);
  const firstField = useRef<HTMLInputElement>(null);
  useKindEvent(setKind);
  useEscapeCloses();
  useEffect(() => {
    firstField.current?.focus();
  }, [kind]);

  const change = (field: keyof QuickAddValues, value: string) => {
    setValues((current) => ({ ...current, [field]: value }));
    setNotice(null);
  };
  const submit = async () => {
    if (!canSubmit(kind, values, busy)) return;
    setBusy(true);
    const outcome = await addition(kind, values, t);
    setBusy(false);
    setNotice(outcome);
    if (outcome.kind === "added") setValues(EMPTY_VALUES);
    firstField.current?.focus();
  };
  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    void submit();
  };

  return (
    <main
      className="flex flex-col gap-3 p-4 h-screen text-text bg-background select-none"
      data-testid={QUICK_ADD_SCREEN}
    >
      <h1 className="text-base font-semibold">{t("quickAdd.title")}</h1>
      <KindTabs
        kind={kind}
        onSelect={(next) => {
          setKind(next);
          setNotice(null);
        }}
      />
      {kind === "replacement" ? (
        <div className="flex flex-col gap-2">
          <Field
            inputRef={firstField}
            name="heard"
            label={t("quickAdd.replacement.heard")}
            value={values.heard}
            onChange={change}
            onKeyDown={onKeyDown}
          />
          <Field
            name="written"
            label={t("quickAdd.replacement.written")}
            value={values.written}
            onChange={change}
            onKeyDown={onKeyDown}
          />
          <p className="text-xs text-mid-gray">
            {t("quickAdd.replacement.hint")}
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          <Field
            inputRef={firstField}
            name="term"
            label={t("quickAdd.vocabulary.term")}
            value={values.term}
            onChange={change}
            onKeyDown={onKeyDown}
          />
          <p className="text-xs text-mid-gray">
            {t("quickAdd.vocabulary.hint")}
          </p>
        </div>
      )}
      <NoticeLine notice={notice} />
      <div className="flex justify-end gap-2 mt-auto">
        <Button
          variant="secondary"
          onClick={() => void getCurrentWindow().close()}
          data-testid={`${QUICK_ADD_SCREEN}-close`}
        >
          {t("quickAdd.close")}
        </Button>
        <Button
          variant="primary"
          disabled={!canSubmit(kind, values, busy)}
          onClick={() => void submit()}
          data-testid={`${QUICK_ADD_SCREEN}-add`}
        >
          {t("quickAdd.add")}
        </Button>
      </div>
    </main>
  );
};

/**
 * Commit the addition of the current tab.
 * @returns the confirmation, or the reasons of the refusal
 */
const addition = async (
  kind: QuickAddKind,
  values: QuickAddValues,
  t: (key: string, options?: Record<string, string>) => string,
): Promise<Notice> => {
  try {
    if (kind === "vocabulary") {
      const term = values.term.trim();
      const result = await commands.quickAddVocabularyTerm(term);
      if (result.status === "ok") {
        return {
          kind: "added",
          text: t("quickAdd.vocabulary.added", { term }),
        };
      }
      return { kind: "failed", failure: quickAddFailure(kind, result.error) };
    }
    const result = await commands.quickAddReplacement(
      values.heard,
      values.written,
    );
    if (result.status === "ok") {
      const text = t("quickAdd.replacement.added", {
        heard: values.heard,
        written: values.written,
      });
      return { kind: "added", text };
    }
    return { kind: "failed", failure: quickAddFailure(kind, result.error) };
  } catch (error) {
    console.error("Quick Add failed:", error);
    return { kind: "failed", failure: quickAddFailure(kind, null) };
  }
};

/** Switch the tab when the tray reopens the window on another kind. */
const useKindEvent = (setKind: (kind: QuickAddKind) => void): void => {
  useEffect(() => {
    const unlisten = listen<QuickAddKind>(QUICK_ADD_KIND_EVENT, (event) =>
      setKind(event.payload),
    );
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [setKind]);
};

/** Escape closes the window. */
const useEscapeCloses = (): void => {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().close();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
};

const KindTabs: React.FC<{
  kind: QuickAddKind;
  onSelect: (kind: QuickAddKind) => void;
}> = ({ kind, onSelect }) => {
  const { t } = useTranslation();
  return (
    <div className="flex gap-1" role="tablist">
      {KINDS.map((option) => (
        <Button
          key={option}
          role="tab"
          aria-selected={option === kind}
          variant={option === kind ? "primary-soft" : "ghost"}
          size="sm"
          onClick={() => onSelect(option)}
          data-testid={`${QUICK_ADD_SCREEN}-tab-${option}`}
        >
          {t(`quickAdd.tabs.${option}`)}
        </Button>
      ))}
    </div>
  );
};

interface FieldProps {
  name: keyof QuickAddValues;
  label: string;
  value: string;
  inputRef?: React.RefObject<HTMLInputElement>;
  onChange: (field: keyof QuickAddValues, value: string) => void;
  onKeyDown: (event: React.KeyboardEvent) => void;
}

const Field: React.FC<FieldProps> = (props) => {
  const { name, label, value, inputRef, onChange, onKeyDown } = props;
  return (
    <Input
      ref={inputRef}
      type="text"
      className="w-full"
      placeholder={label}
      aria-label={label}
      value={value}
      autoComplete="off"
      spellCheck={false}
      data-testid={`${QUICK_ADD_SCREEN}-${name}`}
      onChange={(event) => onChange(name, event.target.value)}
      onKeyDown={onKeyDown}
    />
  );
};

const NoticeLine: React.FC<{ notice: Notice | null }> = ({ notice }) => {
  const { t } = useTranslation();
  if (!notice) return null;
  if (notice.kind === "added") {
    return (
      <Alert variant="success">
        <span data-testid={`${QUICK_ADD_SCREEN}-added`}>{notice.text}</span>
      </Alert>
    );
  }
  const { keys, unchangedKey } = notice.failure;
  const reasons = [...keys, ...(unchangedKey ? [unchangedKey] : [])];
  return (
    <div role="alert" data-testid={`${QUICK_ADD_SCREEN}-refused`}>
      <Alert variant="error">{reasons.map((key) => t(key)).join(" ")}</Alert>
    </div>
  );
};
