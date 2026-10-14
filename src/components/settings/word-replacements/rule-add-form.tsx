/**
 * Add form of the replacement list (contract nodes `replacement-settings.new-recognized`,
 * `.new-replacement`, `.add`). The candidate is checked by the shared Rust validator; Add stays
 * unavailable, with an accessible explanation, until both fields are valid (has_validation); the
 * form is read-only while a Save is in flight.
 */
import React, { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import type { RuleIssue } from "@/bindings";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { isDraftLocked, useReplacementEditor } from "./editor-store";
import { SCREEN_IDS, issueKey } from "./rules-model";

/** Delay before the candidate is checked while typing; one check per pause, not per key. */
const CHECK_DELAY_MS = 200;

interface RuleAddFormProps {
  recognizedRef: React.RefObject<HTMLInputElement>;
  /** Focus the recognized field when the form opens (Add rule of the empty state). */
  focusOnOpen: boolean;
  onAdded: () => void;
}

export const RuleAddForm: React.FC<RuleAddFormProps> = (props) => {
  const { recognizedRef, focusOnOpen, onAdded } = props;
  const { t } = useTranslation();
  const form = useCandidate();
  const hintId = useId();
  useFocusOnOpen(focusOnOpen, recognizedRef);
  const handleSubmit = useAddSubmit(form, recognizedRef, onAdded);
  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2">
        <CandidateField
          form={form}
          field="recognized"
          inputRef={recognizedRef}
          onSubmit={handleSubmit}
        />
        <CandidateField
          form={form}
          field="replacement"
          onSubmit={handleSubmit}
        />
        <AddButton form={form} hintId={hintId} onSubmit={handleSubmit} />
      </div>
      <span id={hintId} className="sr-only">
        {t("settings.replacements.add.needsText")}
      </span>
      <CandidateIssues issues={form.visibleIssues} />
    </div>
  );
};

/**
 * Focus the recognized field when the form opens from the empty state, not on later renders.
 * @param focusOnOpen whether the form was opened by Add rule
 * @param recognizedRef recognized field
 */
const useFocusOnOpen = (
  focusOnOpen: boolean,
  recognizedRef: React.RefObject<HTMLInputElement>,
): void => {
  useEffect(() => {
    if (focusOnOpen) recognizedRef.current?.focus();
  }, [focusOnOpen, recognizedRef]);
};

/**
 * Add the candidate to the draft, then clear the form and return the focus to its first field.
 * @returns the submit handler (ignored until the candidate is ready)
 */
const useAddSubmit = (
  form: Candidate,
  recognizedRef: React.RefObject<HTMLInputElement>,
  onAdded: () => void,
): (() => Promise<void>) => {
  return async () => {
    if (!form.ready) return;
    if (await useReplacementEditor.getState().addRule(form.values)) {
      form.reset();
      onAdded();
      recognizedRef.current?.focus();
    }
  };
};

interface AddButtonProps {
  form: Candidate;
  hintId: string;
  onSubmit: () => Promise<void>;
}

/** Add, unavailable with an accessible explanation until the candidate is valid. */
const AddButton: React.FC<AddButtonProps> = ({ form, hintId, onSubmit }) => {
  const { t } = useTranslation();
  return (
    <Button
      variant="secondary"
      className="flex items-center gap-2 shrink-0"
      disabled={!form.ready}
      aria-describedby={form.ready ? undefined : hintId}
      data-testid={`${SCREEN_IDS.list}-add`}
      onClick={() => void onSubmit()}
    >
      <Plus className="w-4 h-4" aria-hidden="true" />
      {t("settings.replacements.add.button")}
    </Button>
  );
};

type CandidateFieldName = "recognized" | "replacement";

interface Candidate {
  values: { recognized: string; replacement: string };
  ready: boolean;
  /** A Save is in flight: the fields are read-only. */
  locked: boolean;
  visibleIssues: RuleIssue[];
  invalid: (field: CandidateFieldName) => boolean;
  change: (field: CandidateFieldName, value: string) => void;
  touch: (field: CandidateFieldName) => void;
  reset: () => void;
}

/** Both fields hold text. */
const isFilled = (values: Candidate["values"]): boolean =>
  values.recognized.trim() !== "" && values.replacement.trim() !== "";

/** Candidate fields, their shared-validator issues and which fields were left (blurred). */
const useCandidate = (): Candidate => {
  const [values, setValues] = useState({ recognized: "", replacement: "" });
  const [touched, setTouched] = useState({
    recognized: false,
    replacement: false,
  });
  const issues = useCandidateIssues(values);
  const locked = useReplacementEditor(isDraftLocked);
  // A field's error shows once the user leaves it (duplicate AC), and clears as soon as fixed.
  const visibleIssues = (issues ?? []).filter(
    (issue) =>
      issue.field === "rules" || touched[issue.field as CandidateFieldName],
  );
  return {
    values,
    // A Save in flight owns the draft: nothing is added until it ends.
    ready: isFilled(values) && issues?.length === 0 && !locked,
    locked,
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
      readOnly={form.locked}
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
