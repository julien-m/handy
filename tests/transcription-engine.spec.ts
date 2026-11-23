/**
 * Transcription engine tabs (024) in headless Chromium: the real Local / API tabs against the
 * fake IPC of `tests/harness/transcription-engine-fake.ts`. The local tab holds the model list,
 * the API tab activates ChatGPT and shows its sign-in, and with an API engine active the local
 * tab explains the fallback and can switch back. `TRANSCRIPTION_ENGINE_SCREENSHOTS=1` saves
 * review screenshots. Not a native proof. Run with `bun run test:transcription-engine-ui`.
 *
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: is_keyboard_navigable
 */
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/transcription-engine.html";
const SHOTS = ".specs/features/024-transcription-engine/baselines";
const saveShots = process.env.TRANSCRIPTION_ENGINE_SCREENSHOTS === "1";

const changes = (page: Page) =>
  page.evaluate(
    () =>
      (window as unknown as { harness: { backend: { changes: string[] } } })
        .harness.backend.changes,
  );

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 760, height: 460 });
});

test("the local tab shows the model list and marks the active engine", async ({
  page,
}) => {
  await page.goto(HARNESS);
  await expect(page.getByTestId("engine-tab-local")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.getByTestId("engine-tab-local-active-dot")).toBeVisible();
  await expect(page.getByTestId("harness-model-list")).toBeVisible();
  await expect(page.getByTestId("models-fallback-notice")).toHaveCount(0);
  if (saveShots)
    await page.screenshot({
      animations: "disabled",
      path: `${SHOTS}/engine-tab-local.png`,
    });
});

test("activating ChatGPT in the API tab stores it and shows the sign-in", async ({
  page,
}) => {
  await page.goto(HARNESS);
  await page.getByTestId("engine-tab-local").focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByTestId("engine-tab-api")).toBeFocused();
  await expect(page.getByTestId("engine-panel-api")).toBeVisible();
  await page.getByTestId("engine-card-chatgpt-activate").click();
  await expect(page.getByTestId("engine-card-chatgpt-active")).toBeVisible();
  await expect(page.getByTestId("engine-tab-api-active-dot")).toBeVisible();
  await expect(
    page.getByTestId("transcription-engine-signin-signed_in"),
  ).toBeVisible();
  expect(await changes(page)).toEqual(["chatgpt"]);
  if (saveShots)
    await page.screenshot({
      animations: "disabled",
      path: `${SHOTS}/engine-tab-api.png`,
    });
});

test("with ChatGPT active the local tab explains the fallback and switches back", async ({
  page,
}) => {
  await page.goto(`${HARNESS}?engine=chatgpt`);
  await expect(page.getByTestId("engine-tab-api")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await page.getByTestId("engine-tab-local").click();
  await expect(page.getByTestId("engine-tab-local")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.getByTestId("models-fallback-notice")).toBeVisible();
  if (saveShots)
    await page.screenshot({
      animations: "disabled",
      path: `${SHOTS}/engine-tab-local-fallback.png`,
    });
  await page.getByTestId("engine-local-activate").click();
  await expect(page.getByTestId("models-fallback-notice")).toHaveCount(0);
  expect(await changes(page)).toEqual(["local"]);
});

test("ChatGPT without a sign-in shows how to sign in", async ({ page }) => {
  await page.goto(`${HARNESS}?engine=chatgpt&signIn=signed_out`);
  await expect(
    page.getByTestId("transcription-engine-signin-signed_out"),
  ).toBeVisible();
  if (saveShots)
    await page.screenshot({
      animations: "disabled",
      path: `${SHOTS}/engine-signed-out.png`,
    });
});
