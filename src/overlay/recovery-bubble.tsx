// Recovery bubble (010): the transcript Handy kept when it could not insert it, shown in place
// of the live indicator in its own nonactivating window. Reason, two-line preview, a late-read
// warning, macOS drag handle, Copy, Close and the explanation; every control is a native button
// for the pointer and assistive technologies (the window never takes keyboard focus). On macOS
// the whole bubble is the drag source; its buttons other than the handle stay plain clicks.
import { Copy, GripVertical, Info, TriangleAlert, X } from "lucide-react";
import React, { useId } from "react";
import { useTranslation } from "react-i18next";
import type { RecoveryBubbleView } from "@/bindings";
import {
  bubbleTexts,
  type BubbleTexts,
  DRAG_ROLE_ATTRIBUTE,
  type DragOrigin,
  dragOrigin,
  type Point,
  type ReasonIcon,
  type RecoveryScreen,
  recoveryScreen,
} from "./recovery-preview";
import "./recovery-bubble.css";

/** A pointer event on the bubble, reduced to what the drag gesture needs. */
export interface DragPointer {
  kind: "down" | "move" | "up";
  point: Point;
  buttons: number;
  origin: DragOrigin;
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

/** Drag role marker of a control that must stay a plain click. */
const CONTROL = { [DRAG_ROLE_ATTRIBUTE]: "control" };

/** The drag role of the element under the pointer, read from its closest marker. */
const originOf = (target: EventTarget): DragOrigin => {
  const marked =
    target instanceof Element
      ? target.closest(`[${DRAG_ROLE_ATTRIBUTE}]`)
      : null;
  return dragOrigin(marked?.getAttribute(DRAG_ROLE_ATTRIBUTE) ?? null);
};

const pointer =
  (kind: DragPointer["kind"], send: (pointer: DragPointer) => void) =>
  (event: React.PointerEvent) =>
    send({
      kind,
      point: { x: event.clientX, y: event.clientY },
      buttons: event.buttons,
      origin: originOf(event.target),
    });

// @spec FR-003: Whole bubble is the drag source — .specs/features/010-recovery-bubble/spec.md#fr-003
/** Pointer handlers making the whole bubble the drag source (macOS); none without drag. */
const dragSurface = (
  canDrag: boolean,
  onDragPointer: (pointer: DragPointer) => void,
) =>
  canDrag
    ? {
        "data-drag-surface": "true",
        onPointerDown: pointer("down", onDragPointer),
        onPointerMove: pointer("move", onDragPointer),
        onPointerUp: pointer("up", onDragPointer),
      }
    : {};

// @spec FR-003: macOS drag handle of the bubble — .specs/features/010-recovery-bubble/spec.md#fr-003
/** The drag handle: the visible, named drag control; the bubble's gesture starts the drag. */
const DragHandle: React.FC<ScreenProp> = ({ screen }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="rb-pill rb-drag"
      data-testid={`${screen}-drag-handle`}
      data-semantic-id={`${screen}.drag-handle`}
      {...{ [DRAG_ROLE_ATTRIBUTE]: "handle" }}
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
      {...CONTROL}
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
      {...CONTROL}
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
      {...CONTROL}
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
      {...dragSurface(view.can_drag, props.onDragPointer)}
    >
      <Header texts={texts} onClose={props.onClose} screen={screen} />
      <Preview view={view} screen={screen} />
      <LateRead textKey={texts.lateReadKey} screen={screen} />
      {explained && (
        <Explanation id={explanationId} texts={texts} screen={screen} />
      )}
      <div className="rb-row rb-actions">
        {view.can_drag && <DragHandle screen={screen} />}
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
