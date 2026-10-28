// Keyboard focus given back after the control that held it disappears or is disabled (native
// session finding N5): a bar that closes after Save, an inline editor that closes, a dialog whose
// trigger was gone when it opened. WebKit and Chromium then leave the focus on <body>, and the
// next Tab starts over from the top of the window. Each section names a stable point (its title,
// tabIndex -1) that takes the focus back without scrolling.
import { type RefObject, useCallback, useEffect, useRef } from "react";

/**
 * Whether the keyboard focus fell back on the page.
 * @returns true when no element has the focus
 */
export const isFocusLost = (): boolean => {
  const active = document.activeElement;
  return active === null || active === document.body;
};

/**
 * Focuses the first target that is still in the page and enabled, without scrolling.
 * @param targets candidates by preference
 * @returns whether one of them took the focus
 */
export const focusFirstAvailable = (
  targets: ReadonlyArray<HTMLElement | null | undefined>,
): boolean => {
  for (const target of targets) {
    // Skip a target that left the page or is disabled: it cannot hold the focus.
    if (!target || !target.isConnected || target.matches(":disabled")) continue;
    target.focus({ preventScroll: true });
    if (document.activeElement === target) return true;
  }
  return false;
};

/**
 * Watches a section and, when the focus that was inside it falls on <body> because its element
 * was removed or disabled, moves it to `fallback`. A focus the user moved elsewhere, including
 * a click on an empty area, is left alone.
 * Side effects: observes the section's DOM and focus events while mounted.
 * @param fallback stable focus point of the section (its title)
 * @returns ref callback for the section's root element
 */
export const useFocusRescue = (
  fallback: RefObject<HTMLElement | null>,
): ((root: HTMLElement | null) => void) => {
  const cleanup = useRef<(() => void) | null>(null);
  // Kept across detach/attach: a ref callback React calls again (null, then the same element)
  // must not forget that the focus was in the section.
  const memory = useRef<SectionFocus>({ root: null, hadFocus: false });
  useEffect(() => () => cleanup.current?.(), []);
  return useCallback(
    (root: HTMLElement | null) => {
      cleanup.current?.();
      cleanup.current = root
        ? watchSection(root, fallback, memory.current)
        : null;
    },
    [fallback],
  );
};

/** Whether the focus was last inside the watched section. */
interface SectionFocus {
  root: HTMLElement | null;
  hadFocus: boolean;
}

/**
 * Strategy: remember whether the focus is inside the section (focusin), forget it when the user
 * moves it out (focusout to another element, or to nothing while the element is still there and
 * enabled), and after each DOM change of the section move a lost focus to the fallback. Watching
 * the same root again keeps what was remembered and rescues a focus lost in between.
 */
const watchSection = (
  root: HTMLElement,
  fallback: RefObject<HTMLElement | null>,
  memory: SectionFocus,
): (() => void) => {
  const isSameRoot = memory.root === root;
  memory.root = root;
  memory.hadFocus =
    (isSameRoot && memory.hadFocus) || root.contains(document.activeElement);
  const rescue = () => {
    if (memory.hadFocus && isFocusLost())
      focusFirstAvailable([fallback.current]);
  };
  const onFocusIn = () => {
    memory.hadFocus = true;
  };
  const onFocusOut = (event: FocusEvent) => {
    const left = event.target as HTMLElement;
    if (event.relatedTarget instanceof Node) {
      memory.hadFocus = root.contains(event.relatedTarget);
      return;
    }
    // Focus went nowhere: a removed or disabled element is rescued, a click on an empty area
    // (the element is still there and enabled) is the user's choice.
    requestAnimationFrame(() => {
      if (left.isConnected && !left.matches(":disabled"))
        memory.hadFocus = false;
      else rescue();
    });
  };
  const observer = new MutationObserver(rescue);
  observer.observe(root, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["disabled"],
  });
  root.addEventListener("focusin", onFocusIn);
  root.addEventListener("focusout", onFocusOut);
  // A mutation while the section was detached went unobserved.
  rescue();
  return () => {
    observer.disconnect();
    root.removeEventListener("focusin", onFocusIn);
    root.removeEventListener("focusout", onFocusOut);
  };
};
