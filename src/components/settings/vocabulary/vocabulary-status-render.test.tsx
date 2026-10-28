/**
 * The status of a 5,000-term vocabulary over the Whisper budget (native session N7 review): each
 * not-sent group names its first terms and counts the others, grouping stays linear, and every
 * number uses Latin digits like the limit messages (Nepali included). Run with
 * `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { beforeAll, describe, test } from "bun:test";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { renderToStaticMarkup } from "react-dom/server";
import type { ExcludedTerm, VocabularyStatusReport } from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import ne from "@/i18n/locales/ne/translation.json";
import { formatTermCount, VOCABULARY_MAX_TERMS } from "./vocabulary-model";
import { VocabularyStatusView } from "./vocabulary-status";
import {
  NOT_SENT_SHOWN,
  notSentGroups,
  notSentPreview,
} from "./vocabulary-status-model";

const SENT = 50;
const terms = Array.from(
  { length: VOCABULARY_MAX_TERMS },
  (_, index) => `term${index}`,
);
const overBudget: ExcludedTerm[] = terms.slice(SENT).map((term, offset) => ({
  index: SENT + offset,
  term,
  reason: "over_budget",
}));

const report: VocabularyStatusReport = {
  model_id: "small",
  mode: "batch",
  status: "used_during_recognition",
  reason: null,
  mechanism: "whisper_initial_prompt",
  budget: { value: 223, unit: "tokens" },
  terms_sent: terms.slice(0, SENT),
  terms_not_sent: overBudget,
  evidence_kind: "prepared",
};

beforeAll(async () => {
  await i18n.use(initReactI18next).init({
    lng: "en",
    resources: { en: { translation: en }, ne: { translation: ne } },
    interpolation: { escapeValue: false },
  });
});

describe("not-sent terms of a full vocabulary", () => {
  test("a group keeps every term in list order; the status names the first ones", () => {
    const [group] = notSentGroups(overBudget);
    assert.equal(group.terms.length, VOCABULARY_MAX_TERMS - SENT);
    assert.equal(group.terms[0], `term${SENT}`);
    const preview = notSentPreview(group);
    assert.deepEqual(preview.shown, group.terms.slice(0, NOT_SENT_SHOWN));
    assert.equal(preview.hidden, group.terms.length - NOT_SENT_SHOWN);
    const short = { reason: "duplicate" as const, terms: ["a", "b"] };
    assert.deepEqual(notSentPreview(short), { shown: ["a", "b"], hidden: 0 });
  });

  test("the rendered status lists 20 terms and counts the others", () => {
    const markup = renderToStaticMarkup(
      <VocabularyStatusView
        status={{ kind: "ready", report }}
        screen="vocabulary-settings"
        model="Small"
        onRetry={() => undefined}
      />,
    );
    const hidden = VOCABULARY_MAX_TERMS - SENT - NOT_SENT_SHOWN;
    const last = `term${SENT + NOT_SENT_SHOWN - 1}`;
    assert.ok(
      markup.includes(`${last} and ${formatTermCount(hidden, "en")} more.`),
    );
    assert.ok(!markup.includes(`term${SENT + NOT_SENT_SHOWN},`));
    assert.ok(!markup.includes(`term${SENT + NOT_SENT_SHOWN} `));
  });

  test("short groups show no count", () => {
    const one: VocabularyStatusReport = {
      ...report,
      terms_not_sent: overBudget.slice(0, 2),
    };
    const markup = renderToStaticMarkup(
      <VocabularyStatusView
        status={{ kind: "ready", report: one }}
        screen="vocabulary-settings"
        model="Small"
        onRetry={() => undefined}
      />,
    );
    assert.ok(markup.includes(`term${SENT}, term${SENT + 1}.`));
    assert.ok(!markup.includes(" more"));
  });
});

describe("numbers in Latin digits", () => {
  test("Nepali: the counter writes the limit as its messages do", () => {
    const limit = formatTermCount(VOCABULARY_MAX_TERMS, "ne");
    assert.match(limit, /^[0-9,]+$/);
    assert.ok(ne.settings.vocabulary.errors.limit.includes(limit));
  });
});
