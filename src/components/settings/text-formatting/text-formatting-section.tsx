// "Text formatting" section of the Advanced settings page (015, with 016 smart paragraphs): the
// options that adapt the inserted text to the destination field, one independent switch row
// each, drawn from `TEXT_FORMATTING_OPTIONS`.
import React from "react";
import { useTranslation } from "react-i18next";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { useOutputScreen } from "../output-screen";
import { TEXT_FORMATTING_OPTIONS } from "./text-formatting-options";
import { TextFormattingRow } from "./text-formatting-row";

/** The section, with its test id following the settings screen shown. */
export const TextFormattingSection: React.FC = () => {
  const { t } = useTranslation();
  const screen = useOutputScreen();
  return (
    <div data-testid={`${screen}-text-formatting`}>
      <SettingsGroup title={t("settings.advanced.groups.textFormatting")}>
        {TEXT_FORMATTING_OPTIONS.map((option) => (
          <TextFormattingRow key={option.key} option={option} />
        ))}
      </SettingsGroup>
    </div>
  );
};
