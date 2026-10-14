// Tests of the "Return to the dictation's application" view (006 AC-007);
// run with `bun run test:output-settings`.
import { describe, expect, test } from "bun:test";
import { returnToAppView } from "./return-to-app-model";

const KEY = "settings.advanced.returnToApp";

describe("return to the dictation's application", () => {
  test("supported: the toggle shows the stored value", () => {
    expect(returnToAppView({ state: "supported" }, true)).toEqual({
      state: "available",
      checked: true,
      descriptionKey: `${KEY}.description`,
    });
    expect(returnToAppView({ state: "supported" }, false).checked).toBe(false);
  });

  test("gate closed or Linux: unavailable, off, with the platform reason", () => {
    const view = returnToAppView(
      { state: "unsupported", reason: "platform_unsupported" },
      true,
    );
    expect(view).toEqual({
      state: "unavailable",
      checked: false,
      descriptionKey: `${KEY}.unavailablePlatform`,
    });
  });

  test("frontmost app unreadable: unavailable with that reason", () => {
    const view = returnToAppView(
      { state: "unsupported", reason: "frontmost_unreadable" },
      true,
    );
    expect(view.descriptionKey).toBe(`${KEY}.unavailableUnreadable`);
    expect(view.checked).toBe(false);
  });

  test("no destination manager: unavailable", () => {
    expect(returnToAppView(null, true).state).toBe("unavailable");
  });

  test("while loading, the toggle cannot be used yet", () => {
    expect(returnToAppView(undefined, true)).toEqual({
      state: "loading",
      checked: true,
      descriptionKey: `${KEY}.description`,
    });
  });
});
