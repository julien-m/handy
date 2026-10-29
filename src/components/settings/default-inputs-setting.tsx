// "Remembered fields" (013), in the output settings under "Return to the dictation's
// application": the fields and terminals remembered from the badge menu, one per application,
// each with a "Forget" button, and "Forget all". Removing goes through the settings service
// (backend commands); the list is then refreshed. Absent in builds without destination return.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  commands,
  type ApplicationIdentity,
  type DefaultInput,
} from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { useDestinationSupport } from "../../hooks/use-destination-support";
import { Button } from "../ui/Button";
import { SettingContainer } from "../ui/SettingContainer";
import {
  defaultInputsVisible,
  type ForgetPorts,
  identityKey,
  runForget,
  targetDescription,
} from "./default-inputs-model";
import { useOutputScreen } from "./output-screen";

const KEY = "settings.advanced.defaultInputs";

/** Translation function handed to the hook-free view. */
type Translate = (key: string, values?: Record<string, string>) => string;

/** Props of the hook-free list markup. */
export interface DefaultInputsViewProps {
  /** Test-id prefix of the setting (`<screen>-default-inputs`). */
  testId: string;
  entries: DefaultInput[];
  /** A forget failed: the alert is shown. */
  failed: boolean;
  /** A forget is in flight: the buttons are disabled. */
  busy: boolean;
  t: Translate;
  onForget: (identity: ApplicationIdentity) => void;
  onForgetAll: () => void;
}

/** One remembered input: the application, its target and the "Forget" button. */
const DefaultInputRow: React.FC<{
  entry: DefaultInput;
  testId: string;
  busy: boolean;
  t: Translate;
  onForget: (identity: ApplicationIdentity) => void;
}> = ({ entry, testId, busy, t, onForget }) => {
  const target = targetDescription(entry.target);
  return (
    <li
      className="flex items-center justify-between gap-3 py-1"
      data-testid={`${testId}-row`}
      data-kind={entry.target.kind}
    >
      <div className="min-w-0">
        <p className="text-sm font-medium truncate">{entry.display_name}</p>
        <p className="text-xs text-mid-gray truncate">
          {t(target.key, target.values)}
        </p>
      </div>
      <Button
        type="button"
        variant="secondary"
        size="sm"
        disabled={busy}
        aria-label={t(`${KEY}.forgetLabel`, { app: entry.display_name })}
        data-testid={`${testId}-forget`}
        onClick={() => onForget(entry.identity)}
      >
        {t(`${KEY}.forget`)}
      </Button>
    </li>
  );
};

// @spec AC-008: Remembered inputs listed and forgotten — .specs/features/013-default-input-per-app/spec.md#ac-008
/**
 * The list without hooks: the rows, "Forget all" when there is at least one, the empty state
 * otherwise, and the alert after a failed forget.
 * @returns the list markup
 */
export const DefaultInputsView: React.FC<DefaultInputsViewProps> = ({
  testId,
  entries,
  failed,
  busy,
  t,
  onForget,
  onForgetAll,
}) => (
  <div className="flex flex-col gap-2">
    {entries.length === 0 ? (
      <p className="text-sm text-mid-gray" data-testid={`${testId}-empty`}>
        {t(`${KEY}.empty`)}
      </p>
    ) : (
      <ul className="flex flex-col" data-testid={`${testId}-list`}>
        {entries.map((entry) => (
          <DefaultInputRow
            key={identityKey(entry.identity)}
            entry={entry}
            testId={testId}
            busy={busy}
            t={t}
            onForget={onForget}
          />
        ))}
      </ul>
    )}
    {entries.length > 0 && (
      <div>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          disabled={busy}
          aria-label={t(`${KEY}.forgetAllLabel`)}
          data-testid={`${testId}-forget-all`}
          onClick={onForgetAll}
        >
          {t(`${KEY}.forgetAll`)}
        </Button>
      </div>
    )}
    {failed && (
      <p
        className="text-xs text-red-400"
        role="alert"
        data-testid={`${testId}-error`}
      >
        {t(`${KEY}.failed`)}
      </p>
    )}
  </div>
);

/**
 * The backend command of a forget: one application's input, or all of them (null).
 * @param identity the application whose input is forgotten, or null for all
 * @returns the command to run
 */
export const forgetCommand =
  (identity: ApplicationIdentity | null): ForgetPorts["forget"] =>
  () =>
    identity === null
      ? commands.forgetAllDefaultInputs()
      : commands.forgetDefaultInput(identity);

interface DefaultInputsSettingProps {
  grouped?: boolean;
}

/**
 * The "Remembered fields" setting, or nothing in builds that do not offer destination return.
 * @returns the setting row, or null when hidden
 */
export const DefaultInputsSetting: React.FC<DefaultInputsSettingProps> = ({
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const support = useDestinationSupport();
  const screen = useOutputScreen();
  const [failed, setFailed] = useState(false);
  const [busy, setBusy] = useState(false);
  if (!defaultInputsVisible(support)) return null;
  const entries = getSetting("destination_default_inputs") ?? [];
  const testId = `${screen}-default-inputs`;
  const forget = (run: ForgetPorts["forget"]) => {
    setBusy(true);
    void runForget({ forget: run, refresh: refreshSettings }).then((done) => {
      setFailed(!done);
      setBusy(false);
    });
  };
  return (
    <SettingContainer
      title={t(`${KEY}.title`)}
      description={t(`${KEY}.description`)}
      descriptionMode="inline"
      layout="stacked"
      grouped={grouped}
      testId={testId}
    >
      <DefaultInputsView
        testId={testId}
        entries={entries}
        failed={failed}
        busy={busy}
        t={(key, values) => t(key, values)}
        onForget={(identity) => forget(forgetCommand(identity))}
        onForgetAll={() => forget(forgetCommand(null))}
      />
    </SettingContainer>
  );
};
