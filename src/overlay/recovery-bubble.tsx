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
  type RecoveryScreen,
  recoveryScreen,
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

/** Test-id prefix of the screen shown, given to every part of the bubble. */
interface ScreenProp {
  screen: RecoveryScreen;
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
const DragHandle: React.FC<
  Pick<RecoveryBubbleProps, "onDragPointer"> & ScreenProp
> = ({ onDragPointer, screen }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-pill rb-drag"
      data-testid={`${screen}-drag-handle`}
      data-semantic-id={`${screen}.drag-handle`}
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
const CloseButton: React.FC<{ onClose: () => void } & ScreenProp> = ({
  onClose,
  screen,
}) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-close"
      data-testid={`${screen}-close`}
      data-semantic-id={`${screen}.close`}
      aria-label={t("recoveryBubble.actions.close")}
      onClick={onClose}
    >
      <X className="rb-close-icon" aria-hidden="true" />
    </button>
  );
};

/** Reason line with its icon, and the Close button. */
const Header: React.FC<
  { texts: BubbleTexts; onClose: () => void } & ScreenProp
> = ({ texts, onClose, screen }) => {
  const { t } = useTranslation();
  const Icon = ICONS[texts.icon];
  return (
    <div className="rb-row">
      <Icon className={`rb-icon rb-icon-${texts.icon}`} aria-hidden="true" />
      <p
        className="rb-reason"
        data-testid={`${screen}-reason`}
        data-semantic-id={`${screen}.reason`}
        aria-live="polite"
      >
        {t(texts.reasonKey, texts.reasonValues)}
      </p>
      <CloseButton onClose={onClose} screen={screen} />
    </div>
  );
};

/** The first two lines of the transcript, in the direction of its own text. */
const Preview: React.FC<{ view: RecoveryBubbleView } & ScreenProp> = ({
  view,
  screen,
}) => {
  const { t } = useTranslation();
  return (
    <div
      className="rb-preview"
      data-testid={`${screen}-preview`}
      data-semantic-id={`${screen}.preview`}
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
const LateRead: React.FC<{ textKey: string | null } & ScreenProp> = ({
  textKey,
  screen,
}) => {
  const { t } = useTranslation();
  if (textKey === null) return null;
  return (
    <p
      className="rb-late-read"
      data-testid={`${screen}-late-read`}
      role="status"
    >
      {t(textKey)}
    </p>
  );
};

/** The explanation sentences, when shown. */
const Explanation: React.FC<
  { id: string; texts: BubbleTexts } & ScreenProp
> = ({ id, texts, screen }) => {
  const { t } = useTranslation();
  const text = texts.explanation.map((line) => t(line.key, line.values));
  return (
    <div
      id={id}
      className="rb-explanation"
      data-testid={`${screen}-explanation`}
      data-semantic-id={`${screen}.explanation`}
    >
      <p data-testid={`${screen}-explanation-text`}>{text.join(" ")}</p>
    </div>
  );
};

/** Copy, the main action. */
const CopyButton: React.FC<{ onCopy: () => void } & ScreenProp> = ({
  onCopy,
  screen,
}) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-pill rb-strong"
      data-testid={`${screen}-copy`}
      data-semantic-id={`${screen}.copy`}
      onClick={onCopy}
    >
      <Copy className="rb-pill-icon" aria-hidden="true" />
      {t("recoveryBubble.actions.copy")}
    </button>
  );
};

interface WhyProps extends ScreenProp {
  explained: boolean;
  controls: string;
  onToggle: () => void;
}

/** The explanation control: "Why?" or "Hide details". */
const WhyButton: React.FC<WhyProps> = ({
  explained,
  controls,
  onToggle,
  screen,
}) => {
  const { t } = useTranslation();
  const label = explained ? "hideDetails" : "why";
  return (
    <button
      type="button"
      className="rb-link rb-strong"
      data-testid={`${screen}-why`}
      data-semantic-id={`${screen}.why`}
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
  const screen = recoveryScreen(view, explained);
  return (
    <section
      className="rb-bubble"
      data-testid={`${screen}-bubble`}
      data-semantic-id={`${screen}.bubble`}
      data-state={explained ? "explained" : "open"}
      data-reason={view.reason}
      aria-label={t("recoveryBubble.label")}
    >
      <Header texts={texts} onClose={props.onClose} screen={screen} />
      <Preview view={view} screen={screen} />
      <LateRead textKey={texts.lateReadKey} screen={screen} />
      {explained && (
        <Explanation id={explanationId} texts={texts} screen={screen} />
      )}
      <div className="rb-row rb-actions">
        {view.can_drag && (
          <DragHandle onDragPointer={props.onDragPointer} screen={screen} />
        )}
        <CopyButton onCopy={props.onCopy} screen={screen} />
        <WhyButton
          screen={screen}
          explained={explained}
          controls={explanationId}
          onToggle={props.onToggleExplanation}
        />
      </div>
    </section>
  );
};
