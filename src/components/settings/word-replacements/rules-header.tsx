/**
 * Header of the replacement editor: title, description, the List/Text switch and the Import…
 * and Export… buttons (contract nodes `<screen>.rules.title`, `.views`, `.import`, `.export`).
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Download, Upload } from "lucide-react";
import { Button } from "../../ui/Button";
import type { EditorView } from "./editor-store";
import type { ScreenId } from "./rules-model";

interface RulesHeaderProps {
  screen: ScreenId;
  view: EditorView;
  importing: boolean;
  exporting: boolean;
  onView: (view: EditorView) => void;
  onImport: () => void;
  onExport: () => void;
}

export const RulesHeader: React.FC<RulesHeaderProps> = (props) => {
  const { t } = useTranslation();
  return (
    <div className="space-y-3">
      <div>
        <h3
          className="text-sm font-medium"
          data-testid={`${props.screen}-rules-title`}
        >
          {t("settings.replacements.title")}
        </h3>
        <p className="text-sm">{t("settings.replacements.description")}</p>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <ViewSwitch {...props} />
        <FileButtons {...props} />
      </div>
    </div>
  );
};

const ViewSwitch: React.FC<RulesHeaderProps> = ({ screen, view, onView }) => {
  const { t } = useTranslation();
  const option = (value: EditorView) => (
    <Button
      variant={view === value ? "primary-soft" : "ghost"}
      className={view === value ? "" : "text-mid-gray"}
      aria-pressed={view === value}
      data-testid={`${screen}-view-${value}`}
      onClick={() => onView(value)}
    >
      {t(`settings.replacements.views.${value}`)}
    </Button>
  );
  return (
    <div
      role="group"
      aria-label={t("settings.replacements.views.label")}
      className="flex gap-1"
      data-testid={`${screen}-views`}
    >
      {option("list")}
      {option("text")}
    </div>
  );
};

const FileButtons: React.FC<RulesHeaderProps> = (props) => {
  const { t } = useTranslation();
  return (
    <div className="flex gap-2">
      <Button
        variant="secondary"
        className="flex items-center gap-2"
        disabled={props.importing}
        aria-busy={props.importing}
        data-testid={`${props.screen}-import`}
        onClick={props.onImport}
      >
        <Upload className="w-4 h-4" aria-hidden="true" />
        {t("settings.replacements.import")}
      </Button>
      <Button
        variant="secondary"
        className="flex items-center gap-2"
        disabled={props.exporting}
        aria-busy={props.exporting}
        data-testid={`${props.screen}-export`}
        onClick={props.onExport}
      >
        <Download className="w-4 h-4" aria-hidden="true" />
        {t("settings.replacements.export.button")}
      </Button>
    </div>
  );
};
