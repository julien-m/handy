// Pure model of the "Remembered fields" settings list (013): when the list is shown, how a
// remembered target is described, a stable key per application, and the forget actions whose
// failure never throws (the list is refreshed from the settings service after a success).
import type {
  ApplicationIdentity,
  DefaultInput,
  DefaultTarget,
  Result,
} from "@/bindings";
import type { DestinationSupportState } from "../../hooks/use-destination-support";

const KEY = "settings.advanced.defaultInputs";

/** A translated sentence: its key and its interpolation values. */
export interface TranslatedText {
  key: string;
  values: Record<string, string>;
}

/**
 * Whether the list is shown: only in builds offering destination return, once the backend
 * answered (the same rule as "Return to the dictation's application").
 * @param support what the settings know of destination return support
 * @returns true when the backend answered with a support state
 */
export const defaultInputsVisible = (
  support: DestinationSupportState,
): boolean =>
  support !== undefined && support !== null && support !== "not_offered";

/**
 * Name shown for a remembered field: its accessible name, else its placeholder, its DOM id,
 * and its role as the last resort.
 * @param target the remembered field
 * @returns the label of the field
 */
export const fieldLabel = (
  target: Extract<DefaultTarget, { kind: "field" }>,
): string => target.name ?? target.placeholder ?? target.dom_id ?? target.role;

// @spec AC-008: Remembered inputs listed and forgotten — .specs/features/013-default-input-per-app/spec.md#ac-008
/**
 * Description of a remembered target: "Field: <label>" or "Terminal: <workspace>".
 * @param target the remembered target
 * @returns the description sentence
 */
export const targetDescription = (target: DefaultTarget): TranslatedText =>
  target.kind === "field"
    ? { key: `${KEY}.field`, values: { label: fieldLabel(target) } }
    : { key: `${KEY}.terminal`, values: { workspace: target.workspace_name } };

/**
 * Stable key of an application identity (one remembered input per identity).
 * @param identity the application identity
 * @returns the key, unique per identity
 */
export const identityKey = (identity: ApplicationIdentity): string =>
  `${identity.kind}:${identity.value}`;

/** Ports of a forget action: the backend command and the settings refresh. */
export interface ForgetPorts {
  forget: () => Promise<Result<DefaultInput[], string>>;
  refresh: () => Promise<void>;
}

/**
 * Runs a forget command, then refreshes the settings so the list shows the stored state.
 * @param ports the command to run and the settings refresh
 * @returns true when the command succeeded, false when it was refused or failed
 */
export const runForget = async (ports: ForgetPorts): Promise<boolean> => {
  try {
    const result = await ports.forget();
    if (result.status === "error") {
      console.warn("Forget default input refused:", result.error);
      return false;
    }
  } catch (error: unknown) {
    console.warn("Forget default input failed:", error);
    return false;
  }
  // The forget is committed: a failed refresh is not a failed forget (the settings-changed
  // event refreshes the list too).
  await ports.refresh().catch((error: unknown) => {
    console.warn("Settings refresh after forget failed:", error);
  });
  return true;
};
