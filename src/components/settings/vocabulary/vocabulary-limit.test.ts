/**
 * The vocabulary limit (5,000 terms, Julien's decision of 2026-10-28, native session finding N7)
 * is the same in the settings service, the count label and the messages of every locale. Run
 * with `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { VOCABULARY_MAX_TERMS } from "./vocabulary-model";

const here = path.dirname(fileURLToPath(import.meta.url));

describe("vocabulary limit", () => {
  const root = path.resolve(here, "../../../..");
  const LIMIT_MESSAGES = [
    "flag.over_capacity",
    "errors.limit",
    "status.notSent.over_capacity",
    "pasteBox.line.over_capacity",
    "pasteBox.note.overCapacity",
  ];

  test("the count label mirrors MAX_TERMS of the settings service", () => {
    const rust = fs.readFileSync(
      path.join(root, "src-tauri/src/recognition_vocabulary/validation.rs"),
      "utf8",
    );
    const declared = /pub const MAX_TERMS: usize = ([\d_]+);/.exec(rust)?.[1];
    assert.equal(Number(declared?.replace(/_/g, "")), VOCABULARY_MAX_TERMS);
  });

  test("every locale states the limit, formatted for its language", () => {
    const locales = path.join(root, "src/i18n/locales");
    for (const locale of fs.readdirSync(locales)) {
      const vocabulary = JSON.parse(
        fs.readFileSync(path.join(locales, locale, "translation.json"), "utf8"),
      ).settings.vocabulary;
      const limit = new Intl.NumberFormat(locale, {
        numberingSystem: "latn",
      }).format(VOCABULARY_MAX_TERMS);
      for (const key of LIMIT_MESSAGES) {
        const message: string = key
          .split(".")
          .reduce((node, part) => node[part], vocabulary);
        assert.ok(message.includes(limit), `${locale} ${key}: ${message}`);
      }
    }
  });
});
