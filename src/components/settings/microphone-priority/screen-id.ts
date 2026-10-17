// Contract screen shown by the microphone priority editor (feature 001). Every test id of the
// editor carries the prefix of the screen actually shown (`microphone-settings`, `-empty` or
// `-error` in `penflow/flow-ui-contract/contract.json`), so each test id is unique in the tree.
import { createContext, useContext } from "react";

/** Test-id prefix of a contract screen of the editor. */
export type MicrophoneScreen =
  | "microphone-settings"
  | "microphone-settings-empty"
  | "microphone-settings-error";

/** Editor facts that choose the screen. */
export interface MicrophoneScreenInputs {
  loading: boolean;
  /** The committed order changed elsewhere (conflict banner). */
  conflict: boolean;
  /** The last availability check failed (pill and Retry). */
  checkFailed: boolean;
  /** The Add control refused a choice. */
  addRefused: boolean;
  /** Entries of the draft order. */
  count: number;
}

/**
 * Contract screen of the editor.
 * @param inputs editor facts
 * @returns the test-id prefix of the screen shown
 */
export const microphoneScreen = (
  inputs: MicrophoneScreenInputs,
): MicrophoneScreen => {
  // The error screen draws the conflict banner, the failed check and the refused Add: any of them
  // shows that screen.
  if (inputs.conflict || inputs.checkFailed || inputs.addRefused) {
    return "microphone-settings-error";
  }
  // The empty screen is a loaded list with no entry, the draft that removed every entry included;
  // the loading line stays on the base screen.
  if (!inputs.loading && inputs.count === 0) {
    return "microphone-settings-empty";
  }
  return "microphone-settings";
};

/** Screen shown, for every part of the editor; the base screen outside the editor. */
export const MicrophoneScreenContext = createContext<MicrophoneScreen>(
  "microphone-settings",
);

/**
 * Test-id prefix of the screen shown.
 * @returns the screen of the closest editor
 */
export const useMicrophoneScreen = (): MicrophoneScreen =>
  useContext(MicrophoneScreenContext);

/**
 * Instance id of one configured microphone (`instance_id_template` `microphone-{name}`).
 * @param name exact device name
 * @returns the row's instance id
 */
export const microphoneInstanceId = (name: string): string =>
  `microphone-${name}`;
