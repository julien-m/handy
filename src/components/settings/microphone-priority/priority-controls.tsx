// Controls under the priority list (feature 001): Add microphone (enumerated devices; a duplicate
// is refused with its message) and the availability check line — Refresh availability with
// "Checked N s ago", or the failure pill with Retry. As in the validated mockup there is no
// separate Reset control: removing every row empties the draft, which still needs Save.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import Badge from "../../ui/Badge";
import { Dropdown } from "../../ui/Dropdown";
import { secondsSince } from "./availability";
import { PRIORITY_KEY_ROOT, type AddError } from "./draft-state";
import { ADD_FOCUS_KEY } from "./editor-focus";

/** Props of the Add control. */
export interface AddControlProps {
  /** Enumerated device names, or null before a successful listing. */
  enumerated: readonly string[] | null;
  /** The last device listing failed: there is nothing to offer until a Refresh succeeds. */
  listingFailed: boolean;
  addError: AddError | null;
  /** The last choice was added (state "valid" until the next edit). */
  accepted: boolean;
  /** A write is in flight or the settings are not loaded: nothing can be added. */
  busy: boolean;
  onAdd: (name: string) => void;
  onOpen: () => void;
}

/** Props of the availability check line. */
export interface CheckLineProps {
  failed: boolean;
  /** Time of the last device observation, or null before the first read. */
  observedAt: string | null;
  /** Whether "Checked N s ago" applies (a non-empty list). */
  showChecked: boolean;
  onRefresh: () => void;
}

const TICK_MS = 1000; // "Checked N s ago" counts whole seconds.

/**
 * Current time, updated every second while mounted.
 * @returns milliseconds since the epoch
 */
const useNow = (): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), TICK_MS);
    return () => window.clearInterval(timer);
  }, []);
  return now;
};

const LINK_CLASSES =
  "text-sm font-medium text-logo-primary hover:underline cursor-pointer rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary";

/**
 * Visual state of the Add control (has_validation): a refused name is "invalid", an accepted
 * choice "valid" until the next edit, otherwise "empty".
 * @param props Add control props
 * @returns the contract state
 */
const addState = (props: AddControlProps): "invalid" | "valid" | "empty" => {
  if (props.addError !== null) return "invalid";
  return props.accepted ? "valid" : "empty";
};

/**
 * Why the Add control has nothing to offer, as the key of its accessible name.
 * @param props Add control props
 * @returns `refreshFailed` after a failed listing (Refresh availability lists again), `noDevices`
 *   when the listing found nothing, null when there are names to offer or none listed yet
 */
const unavailableKey = (props: AddControlProps): string | null => {
  if (props.listingFailed) return "refreshFailed";
  return props.enumerated !== null && props.enumerated.length === 0
    ? "noDevices"
    : null;
};

/**
 * Add microphone dropdown. A valid choice is added at once and the control empties again; a
 * duplicate stays shown with a red outline (states: empty, invalid, valid). Without any device to
 * add (none found, or the listing failed), the control is disabled and its accessible name says
 * why.
 */
export const AddControl: React.FC<AddControlProps> = (props) => {
  const { t } = useTranslation();
  const names = props.enumerated ?? [];
  const unavailable = unavailableKey(props);
  const noDevices = unavailable !== null;
  const state = addState(props);
  const label = t(`${PRIORITY_KEY_ROOT}.${unavailable ?? "addLabel"}`);
  return (
    <div
      className={`rounded-md ${state === "invalid" ? "ring-1 ring-red-500" : ""}`}
      data-testid="microphone-settings-add"
      data-state={state}
      data-focus-key={ADD_FOCUS_KEY}
    >
      <Dropdown
        options={names.map((name) => ({ value: name, label: name }))}
        selectedValue={props.addError?.name ?? null}
        onSelect={props.onAdd}
        onOpen={props.onOpen}
        placeholder={
          noDevices ? label : t(`${PRIORITY_KEY_ROOT}.addPlaceholder`)
        }
        ariaLabel={label}
        disabled={props.busy || noDevices}
      />
    </div>
  );
};

/** Refusal of the Add control, announced when it appears. */
export const AddErrorText: React.FC<{ error: AddError }> = ({ error }) => {
  const { t } = useTranslation();
  return (
    <p
      role="alert"
      className="text-sm text-red-400"
      data-testid="microphone-settings-error-add-error"
    >
      {t(error.message.key, error.message.params)}
    </p>
  );
};

/** Failed availability check: the pill and Retry. */
const CheckFailed: React.FC<{ onRetry: () => void }> = ({ onRetry }) => {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-3">
      <span data-testid="microphone-settings-error-refresh-error" role="status">
        <Badge variant="error">{t(`${PRIORITY_KEY_ROOT}.refreshFailed`)}</Badge>
      </span>
      <button
        type="button"
        className={LINK_CLASSES}
        data-testid="microphone-settings-error-retry"
        onClick={onRetry}
      >
        {t(`${PRIORITY_KEY_ROOT}.retry`)}
      </button>
    </div>
  );
};

/** Refresh availability, with the age of the last observation. */
export const CheckLine: React.FC<CheckLineProps> = (props) => {
  const { t } = useTranslation();
  const now = useNow();
  if (props.failed) return <CheckFailed onRetry={props.onRefresh} />;
  const seconds =
    props.observedAt === null ? null : secondsSince(props.observedAt, now);
  return (
    <div className="flex items-center gap-3">
      <button
        type="button"
        className={LINK_CLASSES}
        data-testid="microphone-settings-refresh"
        onClick={props.onRefresh}
      >
        {t(`${PRIORITY_KEY_ROOT}.refresh`)}
      </button>
      {props.showChecked && seconds !== null && (
        <span
          className="text-sm text-mid-gray"
          data-testid="microphone-settings-checked"
        >
          {t(`${PRIORITY_KEY_ROOT}.checked`, { seconds: String(seconds) })}
        </span>
      )}
    </div>
  );
};
