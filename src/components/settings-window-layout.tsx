// Layout of the settings window: a fixed sidebar and footer around the one area that scrolls,
// the section content. The window itself never scrolls (native session finding N4).
import React from "react";

interface SettingsWindowLayoutProps {
  /** Writing direction of the app language. */
  direction: "ltr" | "rtl";
  /** Shown above the sidebar and the content (the What's New gate). */
  banner?: React.ReactNode;
  sidebar: React.ReactNode;
  footer: React.ReactNode;
  /** The scrolling content area, read by callers that reset or restore its position. */
  scrollRef?: React.Ref<HTMLDivElement>;
  children: React.ReactNode;
}

/**
 * Settings window frame. The root is clipped to the viewport and the content area is the
 * containing block of its absolutely positioned descendants: a visually hidden element
 * (`sr-only`) far down a section is positioned against it and scrolls with it, instead of
 * stretching the document past the viewport and giving the window a second scrollbar that can
 * scroll the whole frame out of view.
 */
export const SettingsWindowLayout: React.FC<SettingsWindowLayoutProps> = ({
  direction,
  banner,
  sidebar,
  footer,
  scrollRef,
  children,
}) => (
  <div
    dir={direction}
    data-testid="settings-window"
    className="relative h-screen overflow-hidden flex flex-col select-none cursor-default"
  >
    {banner}
    <div className="flex-1 min-h-0 flex overflow-hidden">
      {sidebar}
      <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
        <div
          ref={scrollRef}
          data-testid="settings-window-content"
          className="relative flex-1 overflow-y-auto"
        >
          <div className="flex flex-col items-center p-4 gap-4">{children}</div>
        </div>
      </div>
    </div>
    {footer}
  </div>
);
