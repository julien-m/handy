/**
 * Empty state of the replacement editor (screen `replacement-settings-empty`): Add rule opens
 * the add form, Paste rules as text opens the text view.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Plus, Replace } from "lucide-react";
import { Button } from "../../ui/Button";
import { EXAMPLE_LINE, SCREEN_IDS } from "./rules-model";

const EMPTY_ID = `${SCREEN_IDS.empty}-empty`;

interface RulesEmptyProps {
  /** Add rule, which takes the focus when the last rule is deleted and the list closes. */
  addRef: React.RefObject<HTMLButtonElement>;
  onAdd: () => void;
  onPaste: () => void;
}

export const RulesEmpty: React.FC<RulesEmptyProps> = (props) => {
  const { t } = useTranslation();
  return (
    <div
      className="flex flex-col items-center gap-3 px-4 py-6 text-center border border-mid-gray/20 rounded-lg"
      data-testid={EMPTY_ID}
    >
      <Replace className="w-6 h-6 text-mid-gray" aria-hidden="true" />
      <p className="text-sm font-medium" data-testid={`${EMPTY_ID}-title`}>
        {t("settings.replacements.empty.title")}
      </p>
      <p className="text-sm text-mid-gray" data-testid={`${EMPTY_ID}-body`}>
        {t("settings.replacements.empty.body")} <span>{EXAMPLE_LINE}</span>
      </p>
      <EmptyActions {...props} />
    </div>
  );
};

const EmptyActions: React.FC<RulesEmptyProps> = ({
  addRef,
  onAdd,
  onPaste,
}) => {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap justify-center gap-2">
      <Button
        ref={addRef}
        variant="primary"
        className="flex items-center gap-2"
        data-testid={`${EMPTY_ID}-add`}
        onClick={onAdd}
      >
        <Plus className="w-4 h-4" aria-hidden="true" />
        {t("settings.replacements.add.button")}
      </Button>
      <Button
        variant="secondary"
        data-testid={`${EMPTY_ID}-paste`}
        onClick={onPaste}
      >
        {t("settings.replacements.empty.paste")}
      </Button>
    </div>
  );
};
