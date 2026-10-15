// Recovery bubble (010): the transcript Handy kept when it could not insert it, shown in place
// of the live indicator in its own nonactivating window. Reason, two-line preview, macOS drag
// handle, Copy, Close and the explanation; every control is a native button for the pointer
// and assistive technologies (the window never takes keyboard focus from other apps).
import { Copy, GripVertical, Info, TriangleAlert, X } from "lucide-react";
import React, { useId } from "react";
import { useTranslation } from "react-i18next";
import type { RecoveryBubbleView } from "@/bindings";
import { bubbleTexts, type Point, type ReasonIcon } from "./recovery-preview";
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

interface HandleProps {
  label: string;
  onDragPointer: (pointer: DragPointer) => void;
}

// @spec FR-003: macOS drag handle of the bubble — .specs/features/010-recovery-bubble/spec.md#fr-003
/** The drag handle: a pointer gesture past the threshold starts the native drag. */
const DragHandle: React.FC<HandleProps> = ({ label, onDragPointer }) => (
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
    {label}
  </button>
);

interface HeaderProps {
  icon: ReasonIcon;
  reason: string;
  closeLabel: string;
  onClose: () => void;
}

/** Reason line with its icon, and the Close button. */
const Header: React.FC<HeaderProps> = ({
  icon,
  reason,
  closeLabel,
  onClose,
}) => {
  const Icon = ICONS[icon];
  return (
    <div className="rb-row">
      <Icon className={`rb-icon rb-icon-${icon}`} aria-hidden="true" />
      <p
        className="rb-reason"
        data-testid="recovery-bubble-reason"
        data-semantic-id="recovery-bubble.reason"
        aria-live="polite"
      >
        {reason}
      </p>
      <button
        type="button"
        className="rb-close"
        data-testid="recovery-bubble-close"
        data-semantic-id="recovery-bubble.close"
        aria-label={closeLabel}
        onClick={onClose}
      >
        <X className="rb-close-icon" aria-hidden="true" />
      </button>
    </div>
  );
};

interface ActionsProps {
  canDrag: boolean;
  explained: boolean;
  explanationId: string;
  props: RecoveryBubbleProps;
}

/** Drag handle (macOS), Copy, and the explanation control. */
const Actions: React.FC<ActionsProps> = ({
  canDrag,
  explained,
  explanationId,
  props,
}) => {
  const { t } = useTranslation();
  return (
    <div className="rb-row rb-actions">
      {canDrag && (
        <DragHandle
          label={t("recoveryBubble.actions.drag")}
          onDragPointer={props.onDragPointer}
        />
      )}
      <button
        type="button"
        className="rb-pill rb-strong"
        data-testid="recovery-bubble-copy"
        data-semantic-id="recovery-bubble.copy"
        onClick={props.onCopy}
      >
        <Copy className="rb-pill-icon" aria-hidden="true" />
        {t("recoveryBubble.actions.copy")}
      </button>
      <button
        type="button"
        className="rb-link rb-strong"
        data-testid="recovery-bubble-why"
        data-semantic-id="recovery-bubble.why"
        aria-expanded={explained}
        aria-controls={explained ? explanationId : undefined}
        onClick={props.onToggleExplanation}
      >
        {t(
          explained
            ? "recoveryBubble.actions.hideDetails"
            : "recoveryBubble.actions.why",
        )}
      </button>
    </div>
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
  const explanation = texts.explanation
    .map((line) => t(line.key, line.values))
    .join(" ");
  return (
    <section
      className="rb-bubble"
      data-testid="recovery-bubble-bubble"
      data-semantic-id="recovery-bubble.bubble"
      data-state={explained ? "explained" : "open"}
      data-reason={view.reason}
      aria-label={t("recoveryBubble.label")}
    >
      <Header
        icon={texts.icon}
        reason={t(texts.reasonKey, texts.reasonValues)}
        closeLabel={t("recoveryBubble.actions.close")}
        onClose={props.onClose}
      />
      <div
        className="rb-preview"
        data-testid="recovery-bubble-preview"
        data-semantic-id="recovery-bubble.preview"
        data-truncated={view.truncated ? "true" : "false"}
        role="group"
        aria-label={t("recoveryBubble.previewLabel")}
      >
        <p className="rb-preview-text">{view.preview}</p>
      </div>
      {explained && (
        <div
          id={explanationId}
          className="rb-explanation"
          data-testid="recovery-bubble-explanation-explanation"
          data-semantic-id="recovery-bubble-explanation.explanation"
        >
          <p data-testid="recovery-bubble-explanation-explanation-text">
            {explanation}
          </p>
        </div>
      )}
      <Actions
        canDrag={view.can_drag}
        explained={explained}
        explanationId={explanationId}
        props={props}
      />
    </section>
  );
};
