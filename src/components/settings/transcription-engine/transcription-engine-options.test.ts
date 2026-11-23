// Tests of the transcription engine tabs model (024); run with
// `bun run test:transcription-engine`.
import { describe, expect, test } from "bun:test";
import {
  adjacentTab,
  selectedEngine,
  tabOfEngine,
  usesLocalModel,
} from "./transcription-engine-options";

describe("selectedEngine", () => {
  test("should keep a stored chatgpt engine", () => {
    expect(selectedEngine("chatgpt")).toBe("chatgpt");
  });

  test("should fall back to local for a missing or unknown value", () => {
    expect(selectedEngine(undefined)).toBe("local");
    expect(selectedEngine("soniox")).toBe("local");
  });
});

describe("tabOfEngine", () => {
  test("should place the local engine in the local tab and APIs in the api tab", () => {
    expect(tabOfEngine("local")).toBe("local");
    expect(tabOfEngine("chatgpt")).toBe("api");
  });
});

describe("usesLocalModel", () => {
  test("should lock the model list only when an API engine is active", () => {
    expect(usesLocalModel("local")).toBe(true);
    expect(usesLocalModel("chatgpt")).toBe(false);
  });
});

describe("adjacentTab", () => {
  test("should wrap around in both directions", () => {
    expect(adjacentTab("local", 1)).toBe("api");
    expect(adjacentTab("api", 1)).toBe("local");
    expect(adjacentTab("local", -1)).toBe("api");
  });
});
