// Pure rules of the Automatic Enter setting of the output settings (009 AC-004): which control
// the build shows (the three-value selector when the settings snapshot carries
// `automatic_enter`, gate `destination-return` open; dev's auto-submit control otherwise), the
// options of both selectors and the Penflow test ids of the row, under the output screen shown
// (`output-settings`, or `output-settings-enter-options` while its option list is open).
import type {
  AppSettings,
  AutoSubmitKey,
  AutomaticEnterPolicy,
} from "@/bindings";

/** Screen whose test ids the open option list carries (009). */
const OPTIONS_SCREEN = "output-settings-enter-options";

/**
 * Test ids of the Automatic Enter row under the screen shown (`<screen>-auto-enter-…`).
 * @param screen test-id prefix of the screen shown
 * @returns the row's test ids
 */
export const automaticEnterTestIds = (screen: string) => {
  const base = `${screen}-auto-enter`;
  return {
    row: base,
    title: `${base}-title`,
    mode: `${base}-mode`,
    key: `${base}-key`,
    options: `${base}-options`,
    never: `${base}-never`,
    always: `${base}-always`,
    pinOnly: `${base}-pin-only`,
    note: `${base}-note`,
  };
};

/** Test ids of the options, which only show while the list is open (`-enter-options`). */
const OPTION_TEST_IDS = automaticEnterTestIds(OPTIONS_SCREEN);

const KEY = "settings.advanced.automaticEnter";

/** One option of the Automatic Enter selector. */
export interface AutomaticEnterOption {
  value: AutomaticEnterPolicy;
  labelKey: string;
  /** Subtitle in the open list (none for Never, as in the mockup). */
  descriptionKey: string | null;
  testId: string;
  selected: boolean;
}

/** One option of the key selector. */
export interface EnterKeyOption {
  value: AutoSubmitKey;
  labelKey: string;
}

/** Which Enter control the output settings show. */
export type EnterControl = "automatic_enter" | "auto_submit";

// @spec AC-004: One Automatic Enter control per build — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
/**
 * The control of this build: a gate-open build always carries `automatic_enter` in its settings
 * snapshot (a closed one never does), so exactly one Enter control is shown in either state.
 */
export const enterControl = (
  settings: Pick<AppSettings, "automatic_enter"> | null | undefined,
): EnterControl =>
  settings?.automatic_enter ? "automatic_enter" : "auto_submit";

const POLICIES: {
  value: AutomaticEnterPolicy;
  name: string;
  testId: string;
}[] = [
  { value: "never", name: "never", testId: OPTION_TEST_IDS.never },
  { value: "always", name: "always", testId: OPTION_TEST_IDS.always },
  {
    value: "pin_only",
    name: "pinOnly",
    testId: OPTION_TEST_IDS.pinOnly,
  },
];

/**
 * Never, Always and Only in PIN mode, the stored value marked.
 * @param current Stored policy (`never` while loading).
 */
export const automaticEnterOptions = (
  current: AutomaticEnterPolicy | null | undefined,
): AutomaticEnterOption[] =>
  POLICIES.map(({ value, name, testId }) => ({
    value,
    labelKey: `${KEY}.options.${name}`,
    descriptionKey:
      value === "never" ? null : `${KEY}.options.${name}Description`,
    testId,
    selected: value === (current ?? "never"),
  }));

/**
 * The retained key choice: Enter, Ctrl+Enter, and Cmd+Enter on macOS (Super+Enter elsewhere),
 * with the labels of dev's auto-submit control.
 */
export const enterKeyOptions = (osType: string): EnterKeyOption[] => [
  { value: "enter", labelKey: "settings.advanced.autoSubmit.options.enter" },
  {
    value: "ctrl_enter",
    labelKey: "settings.advanced.autoSubmit.options.ctrlEnter",
  },
  {
    value: "cmd_enter",
    labelKey:
      osType === "macos"
        ? "settings.advanced.autoSubmit.options.cmdEnter"
        : "settings.advanced.autoSubmit.options.superEnter",
  },
];
