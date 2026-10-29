// LiveSpec traceability: @spec(AC-008)
/**
 * Pure model of the "Remembered fields" settings list (013): visibility per destination support,
 * the target description (field label fallbacks, terminal workspace), identity keys, and the
 * forget action (refresh after a success, failure reported, never thrown). Run with
 * `bun run test:output-settings`.
 */
import { describe, expect, test } from "bun:test";
import type { DefaultInput, DefaultTarget } from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import {
  defaultInputsVisible,
  fieldLabel,
  identityKey,
  runForget,
  targetDescription,
} from "./default-inputs-model";

const field = (
  patch: Partial<Extract<DefaultTarget, { kind: "field" }>> = {},
): Extract<DefaultTarget, { kind: "field" }> => ({
  kind: "field",
  role: "AXTextArea",
  subrole: null,
  name: null,
  placeholder: null,
  dom_id: null,
  ...patch,
});
const codex: DefaultInput = {
  identity: { kind: "bundle_id", value: "com.openai.codex" },
  display_name: "Codex",
  target: field({ name: "Work with Codex" }),
};

describe("visibility", () => {
  test("shown once the backend answered, in builds offering destination return", () => {
    expect(defaultInputsVisible({ state: "supported" })).toBe(true);
    expect(
      defaultInputsVisible({
        state: "unsupported",
        reason: "platform_unsupported",
      }),
    ).toBe(true);
    expect(defaultInputsVisible("not_offered")).toBe(false);
    expect(defaultInputsVisible(undefined)).toBe(false);
    expect(defaultInputsVisible(null)).toBe(false);
  });
});

describe("target description (AC-008)", () => {
  test("a field is labelled by its name, else placeholder, DOM id, role", () => {
    const full = field({ name: "N", placeholder: "P", dom_id: "d" });
    expect(fieldLabel(full)).toBe("N");
    expect(fieldLabel({ ...full, name: null })).toBe("P");
    expect(fieldLabel({ ...full, name: null, placeholder: null })).toBe("d");
    expect(fieldLabel(field())).toBe(field().role);
  });
  test("field and terminal have their own sentence with their value", () => {
    const fieldText = targetDescription(codex.target);
    expect(fieldText.values).toEqual({ label: "Work with Codex" });
    const terminal: DefaultTarget = {
      kind: "terminal",
      tool: "cmux",
      workspace_id: "6f1c0d9e-0000-4000-8000-000000000000",
      workspace_name: "roam-radar",
    };
    const terminalText = targetDescription(terminal);
    expect(terminalText.values).toEqual({ workspace: "roam-radar" });
    const sentences = en.settings.advanced.defaultInputs as Record<
      string,
      string
    >;
    const leaf = (key: string) => sentences[key.split(".").pop() as string];
    expect(leaf(fieldText.key)).toContain("{{label}}");
    expect(leaf(terminalText.key)).toContain("{{workspace}}");
  });
  test("identity keys differ by kind and value", () => {
    const keys = [
      identityKey(codex.identity),
      identityKey({ kind: "aumid", value: "com.openai.codex" }),
      identityKey({ kind: "bundle_id", value: "com.cmuxterm.app" }),
    ];
    expect(new Set(keys).size).toBe(keys.length);
  });
});

describe("forget action", () => {
  test("a success refreshes the settings", async () => {
    let refreshed = 0;
    const done = await runForget({
      forget: async () => ({ status: "ok", data: [] }),
      refresh: async () => {
        refreshed += 1;
      },
    });
    expect(done).toBe(true);
    expect(refreshed).toBe(1);
  });
  test("a refusal or a rejected call fails without refreshing nor throwing", async () => {
    let refreshed = 0;
    const refresh = async () => {
      refreshed += 1;
    };
    expect(
      await runForget({
        forget: async () => ({ status: "error", error: "validation_failed" }),
        refresh,
      }),
    ).toBe(false);
    expect(
      await runForget({
        forget: () => Promise.reject(new Error("ipc")),
        refresh,
      }),
    ).toBe(false);
    expect(refreshed).toBe(0);
  });
  test("a failed refresh after a committed forget is still a success", async () => {
    const done = await runForget({
      forget: async () => ({ status: "ok", data: [codex] }),
      refresh: () => Promise.reject(new Error("refresh")),
    });
    expect(done).toBe(true);
  });
});
