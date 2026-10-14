// Destination badge of the recording indicator (feature 006): the application icon and name, or
// "Frontmost app", with the cross that removes the destination of this dictation. Both controls
// are pointer-operated and never take keyboard focus from the user's application.
import { AppWindow } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import { monogram, safeIconSource, type BadgeView } from "./destination-model";

/** Keep a pointer press from focusing the control (the user's app keeps the keyboard). */
export const keepFocus = (event: React.MouseEvent): void =>
  event.preventDefault();

/** The indicator's cross glyph, identical to the cancel button's. */
export const CrossGlyph: React.FC = () => (
  <svg viewBox="0 0 16 16" aria-hidden="true">
    <path
      d="M4 4 L12 12 M12 4 L4 12"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
    />
  </svg>
);

/** Icon of an application: its image when the backend sends one, else a monogram tile. */
export const AppIcon: React.FC<{ name: string; iconRef: string | null }> = ({
  name,
  iconRef,
}) => {
  const source = safeIconSource(iconRef);
  if (source) {
    return <img className="sapp-icon" src={source} alt="" aria-hidden="true" />;
  }
  return (
    <span className="sapp-icon monogram" aria-hidden="true">
      {monogram(name)}
    </span>
  );
};

interface DestinationBadgeProps {
  badge: Exclude<BadgeView, { kind: "hidden" }>;
  menuOpen: boolean;
  onToggleMenu: () => void;
  onRemove: () => void;
}

// @spec FR-003: Badge, accessible name and remove cross — .specs/features/006-destination-application/spec.md#fr-003
/** The badge (opens the destination menu) and, for an application, its remove cross. */
export const DestinationBadge: React.FC<DestinationBadgeProps> = ({
  badge,
  menuOpen,
  onToggleMenu,
  onRemove,
}) => {
  const { t } = useTranslation();
  const name =
    badge.kind === "application"
      ? badge.name
      : t("overlay.destination.frontmostApp");
  return (
    <div className="sdest">
      <button
        type="button"
        className="sdest-btn"
        data-testid="recording-indicator-destination"
        aria-label={t("overlay.destination.label", { name })}
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        disabled={!badge.canChange}
        onMouseDown={keepFocus}
        onClick={onToggleMenu}
      >
        {badge.kind === "application" ? (
          <AppIcon name={badge.name} iconRef={badge.iconRef} />
        ) : (
          <AppWindow className="sdest-window" aria-hidden="true" />
        )}
        <span
          className="sdest-name"
          data-testid="recording-indicator-destination-name"
        >
          {name}
        </span>
      </button>
      {badge.kind === "application" && badge.canChange && (
        <RemoveCross onRemove={onRemove} />
      )}
    </div>
  );
};

const RemoveCross: React.FC<{ onRemove: () => void }> = ({ onRemove }) => {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="sx sdest-remove"
      data-testid="recording-indicator-destination-remove"
      aria-label={t("overlay.destination.remove")}
      onMouseDown={keepFocus}
      onClick={(event) => {
        // The cross never opens the menu (006 AC-004).
        event.stopPropagation();
        onRemove();
      }}
    >
      <CrossGlyph />
    </button>
  );
};
