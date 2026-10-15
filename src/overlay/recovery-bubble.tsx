// Recovery bubble (010): the transcript Handy kept when it could not insert it, shown in place
// of the live indicator in its own nonactivating window. Reason, two-line preview, a late-read
// warning, macOS drag handle, Copy, Close and the explanation; every control is a native button
// for the pointer and assistive technologies (the window never takes keyboard focus).
import { Copy, GripVertical, Info, TriangleAlert, X } from "lucide-react";
import React, { useId } from "react";
import { useTranslation } from "react-i18next";
import type { RecoveryBubbleView } from "@/bindings";
import {
  bubbleTexts,
  type BubbleTexts,
  type Point,
  type ReasonIcon,
} from "./recovery-preview";
import "./recovery-bubble.css";

/** A pointer event on the drag handle, reduced to what the gesture needs. */
export interface DragPointer {
  kind: "down" | "move" | "up";
  point: Point;
  buttons: number;
}

interface RecoveryBubbleProps {
  view: RecoveryBubbleView | null;
  explained: boolean;
  onToggleExplanation: () => void;
  onCopy: () => void;
  onClose: () => void;
  onDragPointer: (pointer: DragPointer) => void;
}

const ICONS: Record<ReasonIcon, typeof Info> = {
  warning: TriangleAlert,
  error: TriangleAlert,
  info: Info,
};

const pointer =
  (kind: DragPointer["kind"], send: (pointer: DragPointer) => void) =>
  (event: React.PointerEvent) =>
    send({
      kind,
      point: { x: event.clientX, y: event.clientY },
      buttons: event.buttons,
    });

// @spec FR-003: macOS drag handle of the bubble — .specs/features/010-recovery-bubble/spec.md#fr-003
/** The drag handle: a pointer gesture past the threshold starts the native drag. */
const DragHandle: React.FC<Pick<RecoveryBubbleProps, "onDragPointer">> = ({
  onDragPointer,
}) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-pill"
      data-testid="recovery-bubble-drag-handle"
      data-semantic-id="recovery-bubble.drag-handle"
      onPointerDown={pointer("down", onDragPointer)}
      onPointerMove={pointer("move", onDragPointer)}
      onPointerUp={pointer("up", onDragPointer)}
    >
      <GripVertical className="rb-pill-icon" aria-hidden="true" />
      {t("recoveryBubble.actions.drag")}
    </button>
  );
};

/** The Close button of the header. */
const CloseButton: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-close"
      data-testid="recovery-bubble-close"
      data-semantic-id="recovery-bubble.close"
      aria-label={t("recoveryBubble.actions.close")}
      onClick={onClose}
    >
      <X className="rb-close-icon" aria-hidden="true" />
    </button>
  );
};

/** Reason line with its icon, and the Close button. */
const Header: React.FC<{ texts: BubbleTexts; onClose: () => void }> = ({
  texts,
  onClose,
}) => {
  const { t } = useTranslation();
  const Icon = ICONS[texts.icon];
  return (
    <div className="rb-row">
      <Icon className={`rb-icon rb-icon-${texts.icon}`} aria-hidden="true" />
      <p
        className="rb-reason"
        data-testid="recovery-bubble-reason"
        data-semantic-id="recovery-bubble.reason"
        aria-live="polite"
      >
        {t(texts.reasonKey, texts.reasonValues)}
      </p>
      <CloseButton onClose={onClose} />
    </div>
  );
};

/** The first two lines of the transcript, in the direction of its own text. */
const Preview: React.FC<{ view: RecoveryBubbleView }> = ({ view }) => {
  const { t } = useTranslation();
  return (
    <div
      className="rb-preview"
      data-testid="recovery-bubble-preview"
      data-semantic-id="recovery-bubble.preview"
      data-truncated={view.truncated ? "true" : "false"}
      role="group"
      aria-label={t("recoveryBubble.previewLabel")}
    >
      <p className="rb-preview-text" dir="auto">
        {view.preview}
      </p>
    </div>
  );
};

/** The late-read warning, visible and announced without opening the explanation. */
const LateRead: React.FC<{ textKey: string | null }> = ({ textKey }) => {
  const { t } = useTranslation();
  if (textKey === null) return null;
  return (
    <p
      className="rb-late-read"
      data-testid="recovery-bubble-late-read"
      role="status"
    >
      {t(textKey)}
    </p>
  );
};

/** The explanation sentences, when shown. */
const Explanation: React.FC<{ id: string; texts: BubbleTexts }> = ({
  id,
  texts,
}) => {
  const { t } = useTranslation();
  const text = texts.explanation.map((line) => t(line.key, line.values));
  return (
    <div
      id={id}
      className="rb-explanation"
      data-testid="recovery-bubble-explanation-explanation"
      data-semantic-id="recovery-bubble-explanation.explanation"
    >
      <p data-testid="recovery-bubble-explanation-explanation-text">
        {text.join(" ")}
      </p>
    </div>
  );
};

/** Copy, the main action. */
const CopyButton: React.FC<{ onCopy: () => void }> = ({ onCopy }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-pill rb-strong"
      data-testid="recovery-bubble-copy"
      data-semantic-id="recovery-bubble.copy"
      onClick={onCopy}
    >
      <Copy className="rb-pill-icon" aria-hidden="true" />
      {t("recoveryBubble.actions.copy")}
    </button>
  );
};

interface WhyProps {
  explained: boolean;
  controls: string;
  onToggle: () => void;
}

/** The explanation control: "Why?" or "Hide details". */
const WhyButton: React.FC<WhyProps> = ({ explained, controls, onToggle }) => {
  const { t } = useTranslation();
  const label = explained ? "hideDetails" : "why";
  return (
    <button
      type="button"
      className="rb-link rb-strong"
      data-testid="recovery-bubble-why"
      data-semantic-id="recovery-bubble.why"
      aria-expanded={explained}
      aria-controls={explained ? controls : undefined}
      onClick={onToggle}
    >
      {t(`recoveryBubble.actions.${label}`)}
    </button>
  );
};

// @spec FR-002: Bubble with preview, reason and actions — .specs/features/010-recovery-bubble/spec.md#fr-002
/** The bubble of `view`; nothing when there is none (closed). */
export const RecoveryBubble: React.FC<RecoveryBubbleProps> = (props) => {
  const { t } = useTranslation();
  const explanationId = useId();
  const { view, explained } = props;
  if (view === null) return null;
  const texts = bubbleTexts(view);
  return (
    <section
      className="rb-bubble"
      data-testid="recovery-bubble-bubble"
      data-semantic-id="recovery-bubble.bubble"
      data-state={explained ? "explained" : "open"}
      data-reason={view.reason}
      aria-label={t("recoveryBubble.label")}
    >
      <Header texts={texts} onClose={props.onClose} />
      <Preview view={view} />
      <LateRead textKey={texts.lateReadKey} />
      {explained && <Explanation id={explanationId} texts={texts} />}
      <div className="rb-row rb-actions">
        {view.can_drag && <DragHandle onDragPointer={props.onDragPointer} />}
        <CopyButton onCopy={props.onCopy} />
        <WhyButton
          explained={explained}
          controls={explanationId}
          onToggle={props.onToggleExplanation}
        />
      </div>
    </section>
  );
};
