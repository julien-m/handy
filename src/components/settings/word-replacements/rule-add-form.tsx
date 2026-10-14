/**
 * Add form of the replacement list (contract nodes `replacement-settings.new-recognized`,
 * `.new-replacement`, `.add`). The candidate is checked by the shared Rust validator; Add stays
 * unavailable, with an accessible explanation, until both fields are valid (has_validation).
 */
import React, { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import type { RuleIssue } from "@/bindings";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { useReplacementEditor } from "./editor-store";
import { SCREEN_IDS, issueKey } from "./rules-model";

/** Delay before the candidate is checked while typing; one check per pause, not per key. */
const CHECK_DELAY_MS = 200;

interface RuleAddFormProps {
  recognizedRef: React.RefObject<HTMLInputElement>;
  /** Focus the recognized field when the form opens (Add rule of the empty state). */
  focusOnOpen: boolean;
  onAdded: () => void;
}

export const RuleAddForm: React.FC<RuleAddFormProps> = ({
  recognizedRef,
  focusOnOpen,
  onAdded,
}) => {
  const { t } = useTranslation();
  const form = useCandidate();
  const hintId = useId();
  useEffect(() => {
    // Runs when the form opens from the empty state, not on later renders.
    if (focusOnOpen) recognizedRef.current?.focus();
  }, [focusOnOpen, recognizedRef]);
  const submit = async () => {
    if (!form.ready) return;
    if (await useReplacementEditor.getState().addRule(form.values)) {
      form.reset();
      onAdded();
      recognizedRef.current?.focus();
    }
  };
  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2">
        <CandidateField
          form={form}
          field="recognized"
          inputRef={recognizedRef}
          onSubmit={submit}
        />
        <CandidateField form={form} field="replacement" onSubmit={submit} />
        <Button
          variant="secondary"
          className="flex items-center gap-2 shrink-0"
          disabled={!form.ready}
          aria-describedby={form.ready ? undefined : hintId}
          data-testid={`${SCREEN_IDS.list}-add`}
          onClick={() => void submit()}
        >
          <Plus className="w-4 h-4" aria-hidden="true" />
          {t("settings.replacements.add.button")}
        </Button>
      </div>
      <span id={hintId} className="sr-only">
        {t("settings.replacements.add.needsText")}
      </span>
      <CandidateIssues issues={form.visibleIssues} />
    </div>
  );
};

type CandidateFieldName = "recognized" | "replacement";

interface Candidate {
  values: { recognized: string; replacement: string };
  ready: boolean;
  visibleIssues: RuleIssue[];
  invalid: (field: CandidateFieldName) => boolean;
  change: (field: CandidateFieldName, value: string) => void;
  touch: (field: CandidateFieldName) => void;
  reset: () => void;
}

/** Candidate fields, their shared-validator issues and which fields were left (blurred). */
const useCandidate = (): Candidate => {
  const [values, setValues] = useState({ recognized: "", replacement: "" });
  const [touched, setTouched] = useState({
    recognized: false,
    replacement: false,
  });
  const issues = useCandidateIssues(values);
  const filled =
    values.recognized.trim() !== "" && values.replacement.trim() !== "";
  // A field's error shows once the user leaves it (duplicate AC), and clears as soon as fixed.
  const visibleIssues = (issues ?? []).filter(
    (issue) =>
      issue.field === "rules" || touched[issue.field as CandidateFieldName],
  );
  return {
    values,
    ready: filled && issues !== null && issues.length === 0,
    visibleIssues,
    invalid: (field) => visibleIssues.some((issue) => issue.field === field),
    change: (field, value) =>
      setValues((current) => ({ ...current, [field]: value })),
    touch: (field) => setTouched((current) => ({ ...current, [field]: true })),
    reset: () => {
      setValues({ recognized: "", replacement: "" });
      setTouched({ recognized: false, replacement: false });
    },
  };
};

/**
 * Shared-validator issues of the candidate, checked after a pause in typing.
 * @returns the issues, or null while a check is pending
 */
const useCandidateIssues = (values: {
  recognized: string;
  replacement: string;
}): RuleIssue[] | null => {
  const [issues, setIssues] = useState<RuleIssue[] | null>([]);
  const draft = useReplacementEditor((state) => state.draft);
  useEffect(() => {
    if (values.recognized === "" && values.replacement === "") {
      setIssues([]);
      return;
    }
    setIssues(null);
    let current = true;
    const timer = window.setTimeout(() => {
      useReplacementEditor
        .getState()
        .checkCandidate(values)
        .then((found) => current && setIssues(found))
        .catch((error: unknown) => {
          console.error("Failed to check the new rule:", error);
          if (current) setIssues(null);
        });
    }, CHECK_DELAY_MS);
    return () => {
      current = false;
      window.clearTimeout(timer);
    };
  }, [values, draft]);
  return issues;
};

interface CandidateFieldProps {
  form: Candidate;
  field: CandidateFieldName;
  inputRef?: React.RefObject<HTMLInputElement>;
  onSubmit: () => Promise<void>;
}

const CandidateField: React.FC<CandidateFieldProps> = ({
  form,
  field,
  inputRef,
  onSubmit,
}) => {
  const { t } = useTranslation();
  const label = t(`settings.replacements.add.${field}`);
  const invalid = form.invalid(field);
  return (
    <Input
      ref={inputRef}
      type="text"
      className={`flex-1 min-w-0 ${invalid ? "border-red-400!" : ""}`}
      placeholder={label}
      aria-label={label}
      aria-invalid={invalid}
      value={form.values[field]}
      data-testid={`${SCREEN_IDS.list}-new-${field}`}
      onChange={(event) => form.change(field, event.target.value)}
      onBlur={() => form.touch(field)}
      onKeyDown={(event) => {
        if (event.key !== "Enter") return;
        event.preventDefault();
        void onSubmit();
      }}
    />
  );
};

const CandidateIssues: React.FC<{ issues: RuleIssue[] }> = ({ issues }) => {
  const { t } = useTranslation();
  if (issues.length === 0) return null;
  return (
    <ul className="text-xs text-red-400 space-y-0.5" role="alert">
      {issues.map((issue) => (
        <li key={`${issue.field}-${issue.kind}`}>
          {t(issueKey(issue.field, issue.kind))}
        </li>
      ))}
    </ul>
  );
};
