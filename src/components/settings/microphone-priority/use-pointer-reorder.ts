// Pointer reorder of the microphone priority rows (feature 001, has_drag_drop). It uses pointer
// events with pointer capture on the row handle instead of HTML drag and drop, which the Tauri
// webview can intercept for file drops; keyboard users reorder with Move up / Move down.
import { useCallback, useEffect, useRef, useState } from "react";
import type { PointerEvent } from "react";

/** Drag in progress: the dragged row and the row it would land on. */
export interface DragState {
  from: number;
  over: number;
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
 * Cancel the drag in progress when Escape is pressed.
 * @param drag drag in progress, or null
 * @param setDrag drag setter
 */
const useEscapeCancel = (drag: DragState | null, setDrag: SetDrag): void => {
  useEffect(() => {
    if (drag === null) return;
    const cancelOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setDrag(null);
    };
    window.addEventListener("keydown", cancelOnEscape);
    return () => window.removeEventListener("keydown", cancelOnEscape);
  }, [drag, setDrag]);
};

/**
 * Pointer handlers of the handle of row `index`.
 * @param index row of the handle
 * @param drag drag in progress, or null
 * @param setDrag drag setter
 * @param locate drop index under a vertical pointer position
 * @param onDrop called with the dragged and target indexes when the drag ends on a new row
 * @returns the handle's pointer props
 */
const handlePropsFor = (
  index: number,
  drag: DragState | null,
  setDrag: SetDrag,
  locate: (pointerY: number) => number,
  onDrop: (from: number, to: number) => void,
): HandleProps => ({
  onPointerDown: (event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    setDrag({ from: index, over: index });
  },
  onPointerMove: (event: PointerEvent<HTMLElement>) => {
    if (drag === null) return;
    const over = locate(event.clientY);
    if (over !== drag.over) setDrag({ from: drag.from, over });
  },
  onPointerUp: () => {
    if (drag !== null && drag.over !== drag.from) onDrop(drag.from, drag.over);
    setDrag(null);
  },
  onPointerCancel: () => setDrag(null),
});

/**
 * Track a pointer drag over the rows and report the drop.
 * Side effects: captures the pointer on the handle and listens for Escape while dragging.
 * @param onDrop called once with the dragged and target indexes when the drag ends on a new row
 * @returns the drag state and the props for rows and handles
 */
export const usePointerReorder = (
  onDrop: (from: number, to: number) => void,
): PointerReorder => {
  const [drag, setDrag] = useState<DragState | null>(null);
  const rows = useRef<Array<HTMLElement | null>>([]);
  useEscapeCancel(drag, setDrag);
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
    handlePropsFor(index, drag, setDrag, locate, onDrop);
  return { drag, rowRef, handleProps };
};
