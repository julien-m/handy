/**
 * Way out of a stalled accessibility request (native session finding N10): when it shows and
 * what it offers. Run with `bun run test:onboarding`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const { AccessibilityWaitingHelp, STALE_WAIT_MS, isWaitStale } = await import(
  "./accessibility-waiting-help"
);

describe("stale wait", () => {
  test("not waiting is never stale", () => {
    expect(isWaitStale(null, 1_000_000)).toBe(false);
  });

  test("stale only once the threshold has passed", () => {
    expect(isWaitStale(1_000, 1_000 + STALE_WAIT_MS - 1)).toBe(false);
    expect(isWaitStale(1_000, 1_000 + STALE_WAIT_MS)).toBe(true);
  });
});

describe("help", () => {
  const markup = renderToStaticMarkup(
    <AccessibilityWaitingHelp
      onOpenSettings={() => undefined}
      onRetry={() => undefined}
    />,
  );

  test("offers Open System Settings and Try again", () => {
    expect(markup.includes("onboarding-accessibility-open-settings")).toBe(
      true,
    );
    expect(markup.includes("onboarding-accessibility-retry")).toBe(true);
    expect(markup.includes("accessibility.openSettings")).toBe(true);
    expect(markup.includes("accessibility.retry")).toBe(true);
  });

  test("says how to clear an entry that is already checked", () => {
    expect(markup.includes("accessibility.alreadyCheckedHint")).toBe(true);
  });
});
