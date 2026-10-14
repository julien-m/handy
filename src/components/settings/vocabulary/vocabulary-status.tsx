/**
 * The single vocabulary status for the selected model (AC-003): one of the five statuses, the
 * terms sent and not sent, or an explicit technical error. Built from the existing Alert and the
 * info-box recipe of the settings pages.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Info } from "lucide-react";
import type { SettingsCommandError, VocabularyStatusReport } from "@/bindings";
import { Alert } from "../../ui/Alert";
import type { StatusState } from "../../../stores/vocabulary-store";
import type { VocabularyScreen } from "./vocabulary-model";
import { RetryButton } from "./vocabulary-notices";
import {
  notSentGroups,
  sentTotal,
  statusText,
  statusTone,
  type StatusTone,
} from "./vocabulary-status-model";

interface StatusBoxProps {
  tone: StatusTone;
  screen: VocabularyScreen;
  title: string;
  body: string | null;
}

/** Neutral facts use the info-box recipe; the other tones use the shared Alert. */
const StatusBox: React.FC<StatusBoxProps> = ({ tone, screen, title, body }) => {
  const titleNode = (
    <span
      className={`block font-medium ${tone === "neutral" ? "text-text" : ""}`}
      data-testid={`${screen}-status-title`}
    >
      {title}
    </span>
  );
  const bodyNode = body ? <span className="block">{body}</span> : null;
  return (
    <div data-testid={`${screen}-status`} data-tone={tone} role="status">
      {tone === "neutral" ? (
        <div className="flex items-start gap-3 p-3 bg-mid-gray/5 rounded-md border border-mid-gray/20 text-sm">
          <Info className="w-5 h-5 shrink-0 mt-0.5 text-mid-gray" />
          <p className="text-mid-gray">
            {titleNode}
            {bodyNode}
          </p>
        </div>
      ) : (
        <Alert variant={tone}>
          {titleNode}
          {bodyNode}
        </Alert>
      )}
    </div>
  );
};

const useReportText = (
  report: VocabularyStatusReport,
  model: string,
): { title: string; body: string } => {
  const { t } = useTranslation();
  const keys = statusText(report);
  const total = sentTotal(report);
  const sent = report.terms_sent.length;
  if (report.status !== "used_during_recognition") {
    return { title: t(keys.title, { model }), body: t(keys.body, { model }) };
  }
  const notSent = notSentGroups(report.terms_not_sent).map((group) =>
    t(`settings.vocabulary.status.notSent.${group.reason}`, {
      terms: group.terms.join(", "),
    }),
  );
  const body = [t(keys.body, { sent, total }), ...notSent].join(" ");
  return { title: t(keys.title, { model }), body };
};

const ReportStatus: React.FC<{
  report: VocabularyStatusReport;
  model: string;
  screen: VocabularyScreen;
}> = ({ report, model, screen }) => {
  const { title, body } = useReportText(report, model);
  return (
    <StatusBox
      tone={statusTone(report)}
      screen={screen}
      title={title}
      body={body}
    />
  );
};

interface VocabularyStatusProps {
  status: StatusState;
  screen: VocabularyScreen;
  /** Display name of the selected model. */
  model: string;
  onRetry: () => void;
}

/** A status read that failed: a durable budget defect, or a transient error with Retry. */
const FailedStatus: React.FC<{
  error: SettingsCommandError | null;
  screen: VocabularyScreen;
  model: string;
  onRetry: () => void;
}> = ({ error, screen, model, onRetry }) => {
  const { t } = useTranslation();
  const root = "settings.vocabulary.status";
  // A declared mechanism without a documented budget is a durable build defect, not one of the
  // five statuses: no retry can clear it, so none is offered.
  if (error?.detail_reason === "context_budget_unverified") {
    const title = t(`${root}.budgetUnverified.title`, { model });
    const body = t(`${root}.budgetUnverified.body`);
    return <StatusBox tone="error" screen={screen} title={title} body={body} />;
  }
  const title = t(`${root}.failed.title`);
  return (
    <div className="flex flex-col items-start gap-2">
      <StatusBox
        tone="error"
        screen={screen}
        title={title}
        body={t(`${root}.failed.body`)}
      />
      <RetryButton onRetry={onRetry} />
    </div>
  );
};

/**
 * Status box of the vocabulary editor.
 * @param props status state, screen prefix for test IDs, model name and retry callback
 */
export const VocabularyStatusView: React.FC<VocabularyStatusProps> = (
  props,
) => {
  const { status, model } = props;
  const { t } = useTranslation();
  const screen = props.screen;
  if (status.kind === "ready") {
    return (
      <ReportStatus report={status.report} model={model} screen={screen} />
    );
  }
  if (status.kind === "failed") {
    return (
      <FailedStatus
        error={status.error}
        screen={screen}
        model={model}
        onRetry={props.onRetry}
      />
    );
  }
  const title = t("settings.vocabulary.status.loading");
  return <StatusBox tone="neutral" screen={screen} title={title} body={null} />;
};
