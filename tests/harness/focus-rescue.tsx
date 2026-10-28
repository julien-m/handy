// Browser harness of `useFocusRescue` (native session finding N5): a section whose root ref is
// written inline, so React detaches and attaches it again on every render, and whose focused
// button removes itself. Served by the Vite dev server for `tests/focus-rescue.spec.ts`.
import React, { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "@/App.css";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { useFocusRescue } from "@/components/ui/focus-return";

// `?webkitFocus`: like WebKit, no focusout reaches the page when the focused element is removed
// or disabled, so only the section's DOM observer can notice the lost focus.
if (new URLSearchParams(window.location.search).has("webkitFocus")) {
  window.addEventListener(
    "focusout",
    (event) => event.stopImmediatePropagation(),
    true,
  );
}

const Section: React.FC = () => {
  const titleRef = useRef<HTMLHeadingElement>(null);
  const rescueRef = useFocusRescue(titleRef);
  const [isShown, setShown] = useState(true);
  return (
    <div ref={(element) => rescueRef(element)} className="p-4">
      <SettingsGroup title="Section" titleRef={titleRef}>
        <div className="p-4">
          {isShown ? (
            <button
              type="button"
              data-testid="leaving-button"
              onClick={() => setShown(false)}
            >
              Done
            </button>
          ) : (
            <p>Saved</p>
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};

const element = document.getElementById("root");
if (element !== null) createRoot(element).render(<Section />);
