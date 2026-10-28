// Keyboard focus of the prompt settings: the first control of a part that
// appears (the editor's name field, an inline fix field, the button that
// opened the editor once it closes) takes the focus, so keyboard and screen
// reader users do not restart from the top of the page.
import { useCallback } from "react";

const FOCUSABLE = "input, textarea, button, [tabindex]";

/**
 * Callback ref that focuses the first focusable element of its node when the
 * node mounts, if `shouldFocus` holds.
 * @param shouldFocus Whether this node takes the focus.
 * @returns A ref callback for the container.
 */
export const useFocusOnMount = <T extends HTMLElement>(
  shouldFocus = true,
): ((node: T | null) => void) =>
  useCallback(
    (node: T | null) => {
      if (!node || !shouldFocus) return;
      const control = node.matches(FOCUSABLE)
        ? node
        : node.querySelector<HTMLElement>(FOCUSABLE);
      control?.focus();
    },
    [shouldFocus],
  );
