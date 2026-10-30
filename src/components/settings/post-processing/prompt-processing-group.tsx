// "AI post-processing" group of the prompt settings (005 AC-003): the
// existing master switch and the default prompt of the AI dictation
// shortcut, "None" or a valid prompt.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { commands } from "@/bindings";
import type { PromptLibraryView } from "@/bindings";
import { settingsErrorKey } from "../../../stores/settings-errors";
import { Dropdown, type DropdownOption } from "../../ui/Dropdown";
import { SettingContainer } from "../../ui/SettingContainer";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { PostProcessingToggle } from "../PostProcessingToggle";
import { NO_PROMPT_VALUE, PROMPT_KEYS } from "./prompt-library-model";

const TEST_ID = "prompt-settings";

interface DefaultPromptSelectProps {
  library: PromptLibraryView | null;
}

/**
 * Options of the default prompt: "None" first, then every prompt; a prompt
 * with issues is listed but cannot be chosen until edited (AC-002).
 */
const useDefaultOptions = (
  library: PromptLibraryView | null,
): DropdownOption[] => {
  const { t } = useTranslation();
  const prompts = library?.prompts ?? [];
  return [
    { value: NO_PROMPT_VALUE, label: t(`${PROMPT_KEYS}.processing.none`) },
    ...prompts.map((prompt) => ({
      value: prompt.id,
      label: prompt.name,
      disabled: prompt.issues.length > 0,
      description:
        prompt.issues.length > 0
          ? t(`${PROMPT_KEYS}.processing.invalidOption`)
          : undefined,
    })),
  ];
};

/**
 * Saves the default prompt; a refusal or a failed command is shown
 * translated (toast), never left unhandled.
 */
const useChooseDefault = () => {
  const { t } = useTranslation();
  const [isSaving, setIsSaving] = useState(false);
  const choose = async (value: string) => {
    setIsSaving(true);
    try {
      const result = await commands.setPostProcessSelectedPrompt(
        value === NO_PROMPT_VALUE ? null : value,
      );
      if (result.status === "error") {
        toast.error(t(settingsErrorKey(result.error)));
      }
    } catch (failure) {
      console.error("Failed to set the default prompt:", failure);
      toast.error(t("errors.settings.generic"));
    } finally {
      setIsSaving(false);
    }
  };
  return { isSaving, choose };
};

// @spec AC-003: Default prompt chosen in settings — .specs/features/005-custom-ai-prompts/spec.md#ac-003
const DefaultPromptSelect: React.FC<DefaultPromptSelectProps> = ({
  library,
}) => {
  const { t } = useTranslation();
  const options = useDefaultOptions(library);
  const { isSaving, choose } = useChooseDefault();
  const title = t(`${PROMPT_KEYS}.processing.defaultTitle`);
  return (
    <SettingContainer
      title={title}
      description={t(`${PROMPT_KEYS}.processing.defaultDescription`)}
      descriptionMode="inline"
      grouped={true}
      testId={`${TEST_ID}-default`}
    >
      <Dropdown
        options={options}
        selectedValue={library?.default_prompt_id ?? NO_PROMPT_VALUE}
        onSelect={(value) => void choose(value)}
        disabled={!library || isSaving}
        ariaLabel={title}
        testId={`${TEST_ID}-default-select`}
        className="min-w-[200px]"
      />
    </SettingContainer>
  );
};

/** Master switch and default prompt. */
export const PromptProcessingGroup: React.FC<DefaultPromptSelectProps> = ({
  library,
}) => {
  const { t } = useTranslation();
  return (
    <div data-testid={`${TEST_ID}-processing`}>
      <SettingsGroup title={t(`${PROMPT_KEYS}.processing.title`)}>
        <PostProcessingToggle
          descriptionMode="inline"
          grouped={true}
          label={t(`${PROMPT_KEYS}.processing.title`)}
          description={t(`${PROMPT_KEYS}.processing.enabledDescription`)}
          testId={`${TEST_ID}-enabled`}
        />
        <DefaultPromptSelect library={library} />
      </SettingsGroup>
    </div>
  );
};
