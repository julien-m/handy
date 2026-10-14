// Reports the size of the indicator stack (prompt pill, destination badge, menus, notices) to
// the native overlay window so it grows to show them and shrinks back afterwards
// (`set_overlay_extent`). The window never goes below the base size of its form.
import { useCallback, useRef } from "react";
import { commands } from "@/bindings";

/** Logical size the overlay window should offer the stack. */
export interface ExtentRequest {
  width: number;
  height: number;
}

/** Room around the stack for its border and sub-pixel rounding, in logical pixels. */
const EXTENT_SLACK = 4;
/** Requests are rounded up to this step so a width transition sends a few requests only. */
const EXTENT_STEP = 8;

const roundUp = (value: number): number =>
  Math.ceil((Math.max(0, value) + EXTENT_SLACK) / EXTENT_STEP) * EXTENT_STEP;

/** The extent to request for a measured stack box. */
export const extentRequest = (
  width: number,
  height: number,
): ExtentRequest => ({
  width: roundUp(width),
  height: roundUp(height),
});

/** True when `next` differs from the last request sent. */
export const extentChanged = (
  last: ExtentRequest | null,
  next: ExtentRequest,
): boolean =>
  last === null || last.width !== next.width || last.height !== next.height;

const sendExtent = (extent: ExtentRequest): void => {
  commands.setOverlayExtent(extent.width, extent.height).catch((error) => {
    console.warn("Could not resize the indicator window", error);
  });
};

/**
 * Callback ref for the stack element: observes its box and reports changes, coalesced per
 * animation frame. Unmounting stops the observation (the next show measures again).
 */
export const useOverlayExtent = (): ((node: HTMLElement | null) => void) => {
  const cleanup = useRef<(() => void) | null>(null);
  return useCallback((node: HTMLElement | null) => {
    cleanup.current?.();
    cleanup.current = null;
    if (!node) return;
    let last: ExtentRequest | null = null;
    let frame = 0;
    const report = () => {
      frame = 0;
      // Layout size, not the transformed box: the pop-in scale must not shrink the request.
      const next = extentRequest(node.offsetWidth, node.offsetHeight);
      if (!extentChanged(last, next)) return;
      last = next;
      sendExtent(next);
    };
    const observer = new ResizeObserver(() => {
      if (frame === 0) frame = requestAnimationFrame(report);
    });
    observer.observe(node);
    report();
    cleanup.current = () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
    };
  }, []);
};
