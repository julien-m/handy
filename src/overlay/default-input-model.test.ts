// LiveSpec traceability: @spec(AC-001)
/**
 * Pure model of the "Remember this field / terminal" entry (013): the label per offer kind and
 * remembered state, the feedback per answer (saved kind or refusal), and the settlement of a
 * remember request that never throws. Run with `bun run test:indicator`.
 */
import { describe, expect, test } from "bun:test";
import type { RememberOffer, RememberRefusal } from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import {
  answerFeedback,
  menuRequestKey,
  offerLabel,
  settleRemember,
} from "./default-input-model";

const offer = (patch: Partial<RememberOffer> = {}): RememberOffer => ({
  kind: "field",
  app_name: "Codex",
  remembered: false,
  ...patch,
});

/** The English text of a dotted key, so every key the model returns is proven to exist. */
const english = (key: string): string => {
  const value = key
    .split(".")
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown> | undefined)?.[part],
      en,
    );
  expect(typeof value).toBe("string");
  return value as string;
};

describe("entry label (AC-001)", () => {
  test("a field names the application; a terminal does not", () => {
    const field = offerLabel(offer());
    expect(field.kind).toBe("field");
    expect(field.values).toEqual({ app: "Codex" });
    expect(english(field.key)).toContain("{{app}}");
    const terminal = offerLabel(offer({ kind: "terminal", app_name: "cmux" }));
    expect(terminal.kind).toBe("terminal");
    expect(terminal.values).toEqual({});
    expect(english(terminal.key)).not.toContain("{{app}}");
  });
  test("an application already remembered gets the 'again' label of its kind", () => {
    for (const kind of ["field", "terminal"] as const) {
      const fresh = offerLabel(offer({ kind }));
      const again = offerLabel(offer({ kind, remembered: true }));
      expect(again.key).toBe(`${fresh.key}Again`);
      expect(english(again.key)).not.toBe(english(fresh.key));
    }
  });
});

describe("remember feedback", () => {
  test("a saved answer names its kind and the application", () => {
    for (const kind of ["field", "terminal"] as const) {
      const saved = offer({ kind, remembered: true });
      const feedback = answerFeedback({ status: "saved", value: saved });
      expect(feedback.values).toEqual({ app: saved.app_name });
      expect(english(feedback.key)).toContain("{{app}}");
    }
    const field = answerFeedback({ status: "saved", value: offer() });
    const terminal = answerFeedback({
      status: "saved",
      value: offer({ kind: "terminal" }),
    });
    expect(field.key).not.toBe(terminal.key);
  });
  test("every refusal has its own translated sentence", () => {
    const refusals: RememberRefusal[] = [
      "stale",
      "not_text_field",
      "password_field",
      "unrecognizable",
      "terminal_unreadable",
      "full",
      "failed",
    ];
    const keys = refusals.map(
      (value) => answerFeedback({ status: "refused", value }).key,
    );
    expect(new Set(keys).size).toBe(refusals.length);
    for (const key of keys) english(key);
  });
});

describe("remember settlement", () => {
  test("saved: the feedback and the offer now remembered", async () => {
    const saved = offer({ remembered: true });
    const outcome = await settleRemember(
      Promise.resolve({ status: "saved", value: saved }),
    );
    expect(outcome.offer).toEqual(saved);
    expect(outcome.feedback).toEqual(
      answerFeedback({ status: "saved", value: saved }),
    );
  });
  test("refused: the offer is unchanged and the reason is said", async () => {
    const answer = { status: "refused", value: "password_field" } as const;
    const outcome = await settleRemember(Promise.resolve(answer));
    expect(outcome.offer).toBeNull();
    expect(outcome.feedback).toEqual(answerFeedback(answer));
  });
  test("a rejected call is the 'could not save' refusal, never a throw", async () => {
    const outcome = await settleRemember(Promise.reject(new Error("ipc")));
    expect(outcome.offer).toBeNull();
    expect(outcome.feedback).toEqual(
      answerFeedback({ status: "refused", value: "failed" }),
    );
  });
  test("requests of another dictation or selection version have another key", () => {
    expect(menuRequestKey(7, 1)).toBe(menuRequestKey(7, 1));
    expect(menuRequestKey(7, 1)).not.toBe(menuRequestKey(7, 2));
    expect(menuRequestKey(7, 1)).not.toBe(menuRequestKey(8, 1));
  });
});
