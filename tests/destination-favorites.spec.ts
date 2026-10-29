// LiveSpec traceability: @spec(AC-001) @spec(AC-003) @spec(AC-004) @spec(AC-005)
/**
 * Destination menu with favorites (012) and the fixed footer (009 AC-001 as amended on
 * 2026-10-29) in headless Chromium: the real menu component and overlay CSS from
 * `tests/harness/destination-menu.tsx`, with many running applications in a short page. Pin,
 * Unpin, "Frontmost app at the end" and the favorites stay visible without scrolling; a star
 * toggles without choosing or closing; the star is a named, pressed toggle reachable with Tab
 * that keeps focus after a keyboard toggle. Not a native window proof.
 * Run with `bunx playwright test tests/destination-favorites.spec.ts`.
 */
import { expect, test, type Locator, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/destination-menu.html";
const MENU = "recording-indicator-pinned-menu";
const SHORT_PAGE = { width: 1280, height: 600 };

const open = async (page: Page, apps = 20): Promise<void> => {
  await page.setViewportSize(SHORT_PAGE);
  await page.goto(`${HARNESS}?apps=${apps}`);
  await expect(page.getByTestId(MENU)).toBeVisible();
};
const recorded = (page: Page) =>
  page.evaluate(
    () =>
      (
        window as unknown as {
          harness: { toggles: number[]; choices: unknown[] };
        }
      ).harness,
  );
const tokenOf = async (locator: Locator): Promise<string> =>
  (await locator.getAttribute("data-favorite-token")) ?? "";

// @spec AC-003: Favorites and footer visible without scrolling — .specs/features/012-destination-favorites/spec.md#ac-003
test("Pin, Unpin, the footer and every favorite are visible without scrolling", async ({
  page,
}) => {
  await open(page);
  const visible = [
    page.getByTestId(`${MENU}-pin`),
    page.getByTestId(`${MENU}-unpin`),
    page.getByTestId(`${MENU}-frontmost-at-end`),
    page.getByTestId(`${MENU}-favorites-label`),
  ];
  for (const locator of visible) {
    await expect(locator).toBeInViewport({ ratio: 1 });
  }
  const favorites = page.getByTestId(`${MENU}-favorite`);
  const count = await favorites.count();
  expect(count).toBeGreaterThan(0);
  for (let index = 0; index < count; index += 1) {
    await expect(favorites.nth(index)).toBeInViewport({ ratio: 1 });
  }
  // Only the running applications scroll; the page itself never does.
  const apps = page.getByTestId(`${MENU}-apps`);
  const overflow = await apps.evaluate(
    (element) => element.scrollHeight > element.clientHeight,
  );
  expect(overflow).toBe(true);
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
  // The footer is the part nearest the badge: Unpin is its last row.
  const unpinBottom = (await page.getByTestId(`${MENU}-unpin`).boundingBox())
    ?.y;
  const badgeTop = (await page.getByTestId("harness-badge").boundingBox())?.y;
  expect(unpinBottom !== undefined && badgeTop !== undefined).toBe(true);
  expect((unpinBottom ?? 0) < (badgeTop ?? 0)).toBe(true);
});

// @spec AC-001: Star toggles without choosing or closing — .specs/features/012-destination-favorites/spec.md#ac-001
test("a star click adds the app as the newest favorite without choosing it", async ({
  page,
}) => {
  await open(page);
  const row = page.getByTestId(`${MENU}-app`).first();
  const name = (await row.locator(".smenu-name").textContent()) ?? "";
  const star = row.locator("xpath=following-sibling::button[1]");
  const token = await tokenOf(star);
  await star.click();
  const newest = page.getByTestId(`${MENU}-favorite`).last();
  await expect(newest.locator(".smenu-name")).toHaveText(name);
  await expect(page.getByTestId(MENU)).toBeVisible();
  const { toggles, choices } = await recorded(page);
  expect(toggles.map(String)).toEqual([token]);
  expect(choices).toEqual([]);
  const starred = page.locator(`[data-favorite-token="${token}"]`);
  await expect(starred).toHaveAttribute("aria-pressed", "true");
  await expect(starred).toHaveAccessibleName(`Remove ${name} from favorites`);
});

// @spec AC-004: Favorite not open is disabled, star still works — .specs/features/012-destination-favorites/spec.md#ac-004
test("a favorite that is not open is disabled and its star removes it", async ({
  page,
}) => {
  await open(page);
  const notes = page
    .getByTestId(`${MENU}-favorite`)
    .filter({ hasText: "Notes" });
  await expect(notes).toBeDisabled();
  await expect(notes).toContainText("Not open");
  const star = notes.locator("xpath=following-sibling::button[1]");
  await star.click();
  await expect(
    page.getByTestId(`${MENU}-favorite`).filter({ hasText: "Notes" }),
  ).toHaveCount(0);
  expect((await recorded(page)).choices).toEqual([]);
});

// @spec AC-005: Star reachable with Tab, focus kept after a toggle — .specs/features/012-destination-favorites/spec.md#ac-005
test("Tab reaches the star after its row and Enter keeps focus on it", async ({
  page,
}) => {
  await open(page, 6);
  const row = page.getByTestId(`${MENU}-app`).first();
  const name = (await row.locator(".smenu-name").textContent()) ?? "";
  await row.focus();
  await page.keyboard.press("Tab");
  const star = page.locator(":focus");
  await expect(star).toHaveAttribute("aria-pressed", "false");
  await expect(star).toHaveAccessibleName(`Add ${name} to favorites`);
  const token = await tokenOf(star);
  await page.keyboard.press("Enter");
  const moved = page.locator(`[data-favorite-token="${token}"]`);
  await expect(moved).toHaveAttribute("aria-pressed", "true");
  await expect(moved).toBeFocused();
  expect((await recorded(page)).choices).toEqual([]);
});
