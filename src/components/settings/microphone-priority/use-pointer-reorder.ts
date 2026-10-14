// Pointer reorder of the microphone priority rows (feature 001, has_drag_drop). It uses pointer
// events with pointer capture on the row handle instead of HTML drag and drop, which the Tauri
// webview can intercept for file drops; keyboard users reorder with Move up / Move down. An
// interrupted drag (capture lost, pointer cancelled, window blurred or hidden, Escape) is dropped
// without reordering, and so is a drag whose row is no longer at its place (its name removed or
// moved by a change saved elsewhere) or that a starting write makes inert.
import { useCallback, useEffect, useRef, useState } from "react";
import type { PointerEvent } from "react";

/** Drag in progress: the pointer that started it, the dragged row and the row it would land on. */
export interface DragState {
  pointerId: number;
  /** Exact name of the dragged row, which must still be at `from` for the drag to go on. */
  name: string;
  from: number;
  over: number;
}

/** What the drag follows besides the pointer. */
export interface ReorderInputs {
  /** Order shown in the list. */
  order: readonly string[];
  /** A write is in flight: the handles are inert and no drag may go on. */
  busy: boolean;
  /** Called once with the dragged and target indexes when the drag ends on a new row. */
  onDrop: (from: number, to: number) => void;
}

/** Handlers and state the list and its handles bind to. */
export interface PointerReorder {
  drag: DragState | null;
  /** Register the element of row `index` (its bounds locate the drop position). */
  rowRef: (index: number) => (element: HTMLElement | null) => void;
  handleProps: (index: number) => {
    onPointerDown: (event: PointerEvent<HTMLElement>) => void;
    onPointerMove: (event: PointerEvent<HTMLElement>) => void;
    onPointerUp: (event: PointerEvent<HTMLElement>) => void;
    onPointerCancel: () => void;
    onLostPointerCapture: () => void;
  };
}

/**
 * Insertion position of a dragged row: the row whose vertical span contains the pointer,
 * clamped to the first and last rows when the pointer leaves the list.
 * @param bounds top and bottom of every row, in list order
 * @param pointerY vertical pointer position, in the same coordinates
 * @returns the target index, or -1 for an empty list
 */
export const dropIndexAt = (
  bounds: ReadonlyArray<{ top: number; bottom: number }>,
  pointerY: number,
): number => {
  if (bounds.length === 0) return -1;
  if (pointerY < bounds[0].top) return 0;
  const index = bounds.findIndex(
    (row) => pointerY >= row.top && pointerY < row.bottom,
  );
  return index >= 0 ? index : bounds.length - 1;
};

type HandleProps = ReturnType<PointerReorder["handleProps"]>;
type SetDrag = (drag: DragState | null) => void;

/**
 * Cancel the drag in progress when it is interrupted: Escape, the window losing focus (Cmd+Tab
 * with the button held) or the page being hidden. A capture lost on the handle and a cancelled
 * pointer are handled by the handle itself.
 * @param dragging whether a drag is in progress
 * @param cancel drops the drag without reordering
 */
const useDragInterruption = (dragging: boolean, cancel: () => void): void => {
  useEffect(() => {
    if (!dragging) return;
    const cancelOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") cancel();
    };
    window.addEventListener("keydown", cancelOnEscape);
    window.addEventListener("blur", cancel);
    document.addEventListener("visibilitychange", cancel);
    return () => {
      window.removeEventListener("keydown", cancelOnEscape);
      window.removeEventListener("blur", cancel);
      document.removeEventListener("visibilitychange", cancel);
    };
  }, [dragging, cancel]);
};

/**
 * Start a drag of row `index` with the primary button, capturing the pointer on the handle.
 * @param index row of the handle
 * @param name exact name of that row
 * @param setDrag drag setter
 * @returns the pointerdown handler
 */
const startDragAt =
  (index: number, name: string, setDrag: SetDrag) =>
  (event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    setDrag({ pointerId: event.pointerId, name, from: index, over: index });
  };

/** What the handles of every row share. */
interface HandleContext {
  /** Drag in progress that is still valid, or null. */
  drag: DragState | null;
  setDrag: SetDrag;
  /** Drop index under a vertical pointer position. */
  locate: (pointerY: number) => number;
  inputs: ReorderInputs;
}

/**
 * Pointer handlers of the handle of row `index`.
 * @param index row of the handle
 * @param context drag, setter, locator and inputs
 * @returns the handle's pointer props
 */
const handlePropsFor = (
  index: number,
  { drag, setDrag, locate, inputs }: HandleContext,
): HandleProps => {
  // Only the pointer that started the drag moves or drops it.
  const ownDrag = (event: PointerEvent<HTMLElement>) =>
    drag !== null && drag.pointerId === event.pointerId ? drag : null;
  return {
    onPointerDown: startDragAt(index, inputs.order[index], setDrag),
    onPointerMove: (event) => {
      const current = ownDrag(event);
      if (current === null) return;
      const over = locate(event.clientY);
      if (over !== current.over) setDrag({ ...current, over });
    },
    onPointerUp: (event) => {
      const current = ownDrag(event);
      if (current === null) return;
      if (current.over !== current.from) {
        inputs.onDrop(current.from, current.over);
      }
      setDrag(null);
    },
    onPointerCancel: () => setDrag(null),
    // Fired after pointerup too, when the drag is already over; otherwise the drag was cut off.
    onLostPointerCapture: () => setDrag(null),
  };
};

/**
 * The drag in progress, or null once it no longer applies: a write started (the handles are
 * inert, so neither pointerup nor lostpointercapture would reach it), or the dragged row is no
 * longer at its place (removed or moved by a change saved elsewhere; a removed handle never
 * reports its lost capture). A stale drag is also cleared, so it never comes back.
 * @param drag drag state
 * @param setDrag drag setter
 * @param inputs current order and write state
 * @returns the valid drag, or null
 */
const useValidDrag = (
  drag: DragState | null,
  setDrag: SetDrag,
  inputs: ReorderInputs,
): DragState | null => {
  const stale =
    drag !== null && (inputs.busy || inputs.order[drag.from] !== drag.name);
  useEffect(() => {
    if (stale) setDrag(null);
  }, [stale, setDrag]);
  return stale ? null : drag;
};

/**
 * Track a pointer drag over the rows and report the drop.
 * Side effects: captures the pointer on the handle; while dragging, listens for Escape, window
 * blur and page visibility to cancel an interrupted drag.
 * @param inputs order shown, write state and drop callback
 * @returns the drag state and the props for rows and handles
 */
export const usePointerReorder = (inputs: ReorderInputs): PointerReorder => {
  const [state, setDrag] = useState<DragState | null>(null);
  const drag = useValidDrag(state, setDrag, inputs);
  const rows = useRef<Array<HTMLElement | null>>([]);
  const cancel = useCallback(() => setDrag(null), []);
  useDragInterruption(drag !== null, cancel);
  const rowRef = useCallback(
    (index: number) => (element: HTMLElement | null) => {
      rows.current[index] = element;
    },
    [],
  );
  const locate = (pointerY: number) =>
    dropIndexAt(
      rows.current
        .filter((row): row is HTMLElement => row !== null)
        .map((row) => row.getBoundingClientRect()),
      pointerY,
    );
  const handleProps = (index: number) =>
    handlePropsFor(index, { drag, setDrag, locate, inputs });
  return { drag, rowRef, handleProps };
};
