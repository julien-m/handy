// State of the recovery window (010): the current projection, pulled when the window loads and
// pushed by the native side (`recovery-bubble-changed`, `recovery-bubble-hidden`), the first
// render of each bubble reported once (that is when the bubble counts as presented), the
// window sized to the bubble, and the actions sent with the exact bubble identity.
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type RecoveryBubbleView } from "@/bindings";
import { syncLanguageFromSettings } from "@/i18n";
import type { DragPointer } from "./recovery-bubble";
import { dragStarts, type Point } from "./recovery-preview";

/** Native event with the projection (`recovery::surface::VIEW_EVENT`). */
export const VIEW_EVENT = "recovery-bubble-changed";
/** Native event of a closed bubble (`recovery::surface::HIDDEN_EVENT`). */
export const HIDDEN_EVENT = "recovery-bubble-hidden";

const warn = (what: string) => (error: unknown) =>
  console.warn(`Recovery bubble: ${what} failed`, error);

/** The projection, from the native side. */
const useProjection = () => {
  const [view, setView] = useState<RecoveryBubbleView | null>(null);
  useEffect(() => {
    void syncLanguageFromSettings();
    commands
      .recoveryBubbleView()
      .then((current) => current && setView(current))
      .catch(warn("reading the bubble"));
    const stops = [
      listen<RecoveryBubbleView>(VIEW_EVENT, (event) => {
        void syncLanguageFromSettings();
        setView(event.payload);
      }),
      listen(HIDDEN_EVENT, () => setView(null)),
    ];
    return () => {
      stops.forEach((stop) => void stop.then((unlisten) => unlisten()));
    };
  }, []);
  return view;
};

// @spec FR-002: Presented only after the first render — .specs/features/010-recovery-bubble/spec.md#fr-002
/** Report the first render of each bubble once, after the browser painted it. */
const useRenderedReport = (view: RecoveryBubbleView | null) => {
  const reported = useRef<number | null>(null);
  useEffect(() => {
    if (view === null || reported.current === view.bubble_id) return;
    reported.current = view.bubble_id;
    const bubble = view.bubble_id;
    requestAnimationFrame(() => {
      commands
        .recoveryBubbleRendered(bubble)
        .catch(warn("reporting the render"));
    });
  }, [view]);
};

/** Size the native window to the bubble element. */
const useWindowSize = () => {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = ref.current;
    if (element === null) return;
    const observer = new ResizeObserver(() => {
      const { width, height } = element.getBoundingClientRect();
      if (width > 0 && height > 0) {
        commands
          .recoveryBubbleResize(Math.ceil(width), Math.ceil(height))
          .catch(warn("sizing the window"));
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return ref;
};

/** The drag gesture of the handle: past the threshold, ask the native side to drag. */
const useDragGesture = (view: RecoveryBubbleView | null) => {
  const start = useRef<Point | null>(null);
  return useCallback(
    (pointer: DragPointer) => {
      if (pointer.kind !== "move") {
        start.current = pointer.kind === "down" ? pointer.point : null;
        return;
      }
      if (
        view === null ||
        !dragStarts(start.current, pointer.point, pointer.buttons)
      ) {
        return;
      }
      start.current = null;
      commands
        .recoveryBubbleBeginDrag(view.bubble_id)
        .catch(warn("starting the drag"));
    },
    [view],
  );
};

/** Everything the recovery window renders and does. */
export const useRecoveryBubble = () => {
  const view = useProjection();
  const [explained, setExplained] = useState(false);
  const bubbleId = view?.bubble_id ?? null;
  useEffect(() => setExplained(false), [bubbleId]);
  useRenderedReport(view);
  const ref = useWindowSize();
  const onDragPointer = useDragGesture(view);
  const onCopy = useCallback(() => {
    if (view === null) return;
    commands
      .recoveryBubbleCopy(view.bubble_id, view.destination_generation)
      .catch(warn("copying"));
  }, [view]);
  const onClose = useCallback(() => {
    if (view === null) return;
    commands.recoveryBubbleClose(view.bubble_id).catch(warn("closing"));
  }, [view]);
  const onToggleExplanation = useCallback(
    () => setExplained((shown) => !shown),
    [],
  );
  return {
    view,
    explained,
    ref,
    onCopy,
    onClose,
    onDragPointer,
    onToggleExplanation,
  };
};
