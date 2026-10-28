/**
 * "Permissions Required" screen in headless Chromium, against the fake Tauri IPC of
 * `tests/harness/accessibility-onboarding-fake.ts` (native session finding N10): a request that
 * stays "Waiting…" offers Open System Settings, Try again and the hint for an entry already
 * checked, and the permission is checked again when the window takes the focus back. Not a native
 * window proof: the real System Settings pane and the macOS request are not observed here.
 */
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/accessibility-onboarding.html";
const HELP = "onboarding-accessibility-help";

interface Backend {
  accessibility: boolean;
  accessibilityChecks: number;
  accessibilityRequests: number;
  openedUrls: string[];
}

const backend = (page: Page): Promise<Backend> =>
  page.evaluate(
    () =>
      (window as unknown as { harness: { backend: Backend } }).harness.backend,
  );

const grantAccessibility = (page: Page) =>
  page.evaluate(() => {
    (
      window as unknown as { harness: { backend: Backend } }
    ).harness.backend.accessibility = true;
  });

const startWaiting = async (page: Page): Promise<void> => {
  await page.clock.install();
  await page.goto(HARNESS);
  await page.getByRole("button", { name: "Grant Permission" }).click();
  await expect(page.getByText("Waiting...")).toBeVisible();
};

test("the way out appears only after a few seconds of waiting", async ({
  page,
}) => {
  await startWaiting(page);
  await expect(page.getByTestId(HELP)).toHaveCount(0);
  await page.clock.runFor(5_000);
  const help = page.getByTestId(HELP);
  await expect(help).toBeVisible();
  await expect(help).toContainText(
    "If Handy is already checked, remove it, then add it again.",
  );
});

test("Open System Settings opens Privacy & Security › Accessibility", async ({
  page,
}) => {
  await startWaiting(page);
  await page.clock.runFor(5_000);
  await page.getByTestId("onboarding-accessibility-open-settings").click();
  await expect
    .poll(async () => (await backend(page)).openedUrls)
    .toEqual([
      "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    ]);
});

test("Try again asks macOS again and restarts the wait", async ({ page }) => {
  await startWaiting(page);
  await page.clock.runFor(5_000);
  await page.getByTestId("onboarding-accessibility-retry").click();
  await expect
    .poll(async () => (await backend(page)).accessibilityRequests)
    .toBe(2);
  await expect(page.getByTestId(HELP)).toHaveCount(0);
});

test("a permission granted elsewhere is seen when the window takes the focus back", async ({
  page,
}) => {
  await startWaiting(page);
  await grantAccessibility(page);
  const checks = (await backend(page)).accessibilityChecks;
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect
    .poll(async () => (await backend(page)).accessibilityChecks)
    .toBeGreaterThan(checks);
  await expect(page.getByText("All set!")).toBeVisible();
});

test("checks that overlap run once and complete the onboarding once", async ({
  page,
}) => {
  await startWaiting(page);
  await grantAccessibility(page);
  const checks = (await backend(page)).accessibilityChecks;
  // The page focus, the Tauri focus event and the polling can all ask at the same moment.
  await page.evaluate(() => {
    window.dispatchEvent(new Event("focus"));
    window.dispatchEvent(new Event("focus"));
    window.dispatchEvent(new Event("focus"));
  });
  await expect(page.getByText("All set!")).toBeVisible();
  expect((await backend(page)).accessibilityChecks).toBe(checks + 1);
  await page.clock.runFor(2_000);
  const completions = await page.evaluate(
    () =>
      (window as unknown as { harness: { state: { completions: number } } })
        .harness.state.completions,
  );
  expect(completions).toBe(1);
});
