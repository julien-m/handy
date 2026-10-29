// LiveSpec traceability: @spec(AC-008)
/**
 * Rendered "Remembered fields" setting (013): hidden without destination return, the rows with
 * the application and its target, the named "Forget" buttons and "Forget all", the empty state,
 * the alert after a failed forget, and the backend command each button runs. Run with
 * `bun run test:output-settings`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { isValidElement, type ReactElement, type ReactNode } from "react";
import type { ApplicationIdentity, DefaultInput } from "@/bindings";
import type { DestinationSupportState } from "../../hooks/use-destination-support";

/** Test translation: the key followed by its values, so both are visible in the markup. */
const t = (key: string, values?: Record<string, string>): string =>
  [key, ...Object.values(values ?? {})].join(" ");
mock.module("react-i18next", () => ({ useTranslation: () => ({ t }) }));

let support: DestinationSupportState = { state: "supported" };
mock.module("../../hooks/use-destination-support", () => ({
  useDestinationSupport: () => support,
}));
let stored: Record<string, unknown> = {};
mock.module("../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) => stored[key],
    refreshSettings: async () => undefined,
  }),
}));
const invoked: { command: string; identity?: ApplicationIdentity }[] = [];
mock.module("@/bindings", () => ({
  commands: {
    forgetDefaultInput: async (identity: ApplicationIdentity) => {
      invoked.push({ command: "forgetDefaultInput", identity });
      return { status: "ok", data: [] };
    },
    forgetAllDefaultInputs: async () => {
      invoked.push({ command: "forgetAllDefaultInputs" });
      return { status: "ok", data: [] };
    },
  },
}));

const { DefaultInputsSetting, DefaultInputsView, forgetCommand } = await import(
  "./default-inputs-setting"
);
const { targetDescription } = await import("./default-inputs-model");

const KEY = "settings.advanced.defaultInputs";
const ID = "output-settings-default-inputs";

const codex: DefaultInput = {
  identity: { kind: "bundle_id", value: "com.openai.codex" },
  display_name: "Codex",
  target: {
    kind: "field",
    role: "AXTextArea",
    subrole: null,
    name: "Work with Codex",
    placeholder: null,
    dom_id: null,
  },
};
const cmux: DefaultInput = {
  identity: { kind: "bundle_id", value: "com.cmuxterm.app" },
  display_name: "cmux",
  target: {
    kind: "terminal",
    tool: "cmux",
    workspace_id: "6f1c0d9e-0000-4000-8000-000000000000",
    workspace_name: "roam-radar",
  },
};

const renderSetting = (entries: DefaultInput[] | undefined): string => {
  stored = { destination_default_inputs: entries };
  return renderToStaticMarkup(<DefaultInputsSetting grouped />);
};

type Props = Record<string, unknown> & { children?: ReactNode };
/** Elements of the tree, hook-free function components expanded, in document order. */
const elements = (node: ReactNode): ReactElement<Props>[] => {
  if (Array.isArray(node)) return node.flatMap(elements);
  if (!isValidElement<Props>(node)) return [];
  if (typeof node.type === "function") {
    const component = node.type as (props: Props) => ReactNode;
    return elements(component(node.props));
  }
  return [node, ...elements(node.props.children)];
};
const byTestId = (node: ReactNode, testId: string) =>
  elements(node).filter((element) => element.props["data-testid"] === testId);

const viewTree = (entries: DefaultInput[], onForget = (_: unknown) => {}) => {
  const calls = { forgetAll: 0 };
  const tree = (
    <DefaultInputsView
      testId={ID}
      entries={entries}
      failed={false}
      busy={false}
      t={t}
      onForget={onForget}
      onForgetAll={() => (calls.forgetAll += 1)}
    />
  );
  return { tree, calls };
};

describe("remembered fields setting (AC-008)", () => {
  test("hidden in builds without destination return or before the answer", () => {
    for (const hidden of ["not_offered", undefined, null] as const) {
      support = hidden;
      expect(renderSetting([codex])).toBe("");
    }
    support = { state: "supported" };
  });
  test("one row per application with its target, and Forget all", () => {
    const markup = renderSetting([codex, cmux]);
    expect(markup).toContain(`data-testid="${ID}-title"`);
    expect(markup).toContain(`${KEY}.title`);
    const rows = markup.match(new RegExp(`data-testid="${ID}-row"`, "g"));
    expect(rows?.length).toBe(2);
    for (const entry of [codex, cmux]) {
      const target = targetDescription(entry.target);
      expect(markup).toContain(`>${entry.display_name}<`);
      expect(markup).toContain(`>${t(target.key, target.values)}<`);
      expect(markup).toContain(
        `aria-label="${t(`${KEY}.forgetLabel`, { app: entry.display_name })}"`,
      );
    }
    expect(markup).toContain(`data-testid="${ID}-forget-all"`);
    expect(markup).toContain(`aria-label="${t(`${KEY}.forgetAllLabel`)}"`);
    expect(markup).not.toContain(`data-testid="${ID}-empty"`);
    expect(markup).not.toContain('role="alert"');
  });
  test("empty list: the empty state, no Forget all", () => {
    for (const entries of [[], undefined]) {
      const markup = renderSetting(entries);
      expect(markup).toContain(`data-testid="${ID}-empty"`);
      expect(markup).toContain(`${KEY}.empty`);
      expect(markup).not.toContain(`data-testid="${ID}-forget-all"`);
    }
  });
  test("a failed forget is announced as an alert", () => {
    const markup = renderToStaticMarkup(
      <DefaultInputsView
        testId={ID}
        entries={[codex]}
        failed
        busy
        t={t}
        onForget={() => undefined}
        onForgetAll={() => undefined}
      />,
    );
    expect(markup).toMatch(/role="alert" data-testid="[^"]+-error">/);
    expect(markup).toContain(`${KEY}.failed`);
    expect(markup).toMatch(/disabled=""[^>]*data-testid="[^"]+-forget"/);
  });
  test("Forget hands its row's identity; Forget all asks for all", () => {
    const forgotten: unknown[] = [];
    const { tree, calls } = viewTree([codex, cmux], (identity) =>
      forgotten.push(identity),
    );
    const forgets = byTestId(tree, `${ID}-forget`);
    expect(forgets.length).toBe(2);
    (forgets[1].props.onClick as () => void)();
    expect(forgotten).toEqual([cmux.identity]);
    const [all] = byTestId(tree, `${ID}-forget-all`);
    (all.props.onClick as () => void)();
    expect(calls.forgetAll).toBe(1);
  });
  test("each forget runs its backend command with the identity", async () => {
    invoked.length = 0;
    await forgetCommand(codex.identity)();
    await forgetCommand(null)();
    expect(invoked).toEqual([
      { command: "forgetDefaultInput", identity: codex.identity },
      { command: "forgetAllDefaultInputs" },
    ]);
  });
});
