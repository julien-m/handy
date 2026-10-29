// "Remember this field for <app>" / "Remember this terminal" (013), in the fixed footer of the
// badge menu: remembers where Handy pastes for the destination application when its active
// element is not a text field. Pointer presses never take keyboard focus from the user's
// application; the menu stays open and the destination never changes.
import { SquareTerminal, TextCursorInput } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import type { DefaultInputKind } from "@/bindings";
import { keepFocus } from "./destination-badge";
import { offerLabel } from "./default-input-model";
import { useMenuId } from "./destination-menu-rows";
import { useDefaultInput } from "./use-default-input";

/** Props of the hook-free entry markup. */
export interface DefaultInputViewProps {
  /** Test id of the destination menu (`<screen>-menu`). */
  menuId: string;
  kind: DefaultInputKind;
  label: string;
  /** Translated feedback of the last remember, or null. */
  feedback: string | null;
  onRemember: () => void;
}

/**
 * The entry and its feedback, without hooks: what the menu renders for one offer.
 * @returns the menu item followed by its status line when there is feedback
 */
export const DefaultInputView: React.FC<DefaultInputViewProps> = ({
  menuId,
  kind,
  label,
  feedback,
  onRemember,
}) => {
  const Glyph = kind === "terminal" ? SquareTerminal : TextCursorInput;
  return (
    <>
      <button
        type="button"
        role="menuitem"
        className="smenu-item"
        data-testid={`${menuId}-remember-input`}
        data-kind={kind}
        onMouseDown={keepFocus}
        onClick={onRemember}
      >
        <Glyph className="smenu-glyph" aria-hidden="true" />
        <span className="smenu-name">{label}</span>
      </button>
      {feedback && (
        <p
          className="smenu-feedback"
          role="status"
          data-testid={`${menuId}-remember-feedback`}
        >
          {feedback}
        </p>
      )}
    </>
  );
};

// @spec AC-001: Remember entry in the badge menu — .specs/features/013-default-input-per-app/spec.md#ac-001
/**
 * The remember entry of the open menu; nothing when the backend offers none (no application
 * destination, gate closed, Windows, Linux).
 * @returns the entry, or null without an offer
 */
export const DefaultInputEntry: React.FC<{
  dictationId: number;
  selectionVersion: number;
}> = ({ dictationId, selectionVersion }) => {
  const { t } = useTranslation();
  const menuId = useMenuId();
  const { offer, feedback, remember } = useDefaultInput(
    dictationId,
    selectionVersion,
  );
  if (!offer) return null;
  const label = offerLabel(offer);
  return (
    <DefaultInputView
      menuId={menuId}
      kind={label.kind}
      label={t(label.key, label.values)}
      feedback={feedback ? t(feedback.key, feedback.values) : null}
      onRemember={remember}
    />
  );
};
