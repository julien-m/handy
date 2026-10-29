// LiveSpec traceability: @spec(AC-001) @spec(AC-003) @spec(AC-004) @spec(AC-005)
// 009 decision no_dictation_only_hint and 012 decision compact_row (N29, 2026-10-29).
/**
 * Destination menu with favorites (012) and the fixed footer (009 AC-001 as amended on
 * 2026-10-29) in headless Chromium: the real menu component and overlay CSS from
 * `tests/harness/destination-menu.tsx`, with many running applications in a short page. Pin,
 * Unpin, "Frontmost app at the end" and the favorites stay visible without scrolling; a star
 * toggles without choosing or closing; the star is a named, pressed toggle reachable with Tab
 * that keeps focus after a keyboard toggle; realistic application names are shown in full beside
 * the star (compact row, no "This dictation only" hint). Not a native window proof.
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

/** Realistic application names of Julien's N29 capture. */
const REAL_NAMES = [
  "Raindrop.io",
  "Google Chrome",
  "Stream Deck Guardian",
  "Free Download Manager",
];
/** Names up to about 20 characters must be shown in full. */
const READABLE_LENGTH = 21;

// @spec AC-001: Compact row, full app name readable — .specs/features/012-destination-favorites/spec.md#ac-001
test("realistic app names are shown in full beside the star, with no hint", async ({
  page,
}) => {
  await page.setViewportSize(SHORT_PAGE);
  const names = encodeURIComponent(REAL_NAMES.join(","));
  await page.goto(`${HARNESS}?apps=${REAL_NAMES.length + 2}&names=${names}`);
  const menu = page.getByTestId(MENU);
  await expect(menu).toBeVisible();
  // The production width of the menu, read from the overlay stylesheet, not restated here.
  const widths = await menu.evaluate((element) => ({
    rendered: element.getBoundingClientRect().width,
    token: getComputedStyle(element).getPropertyValue("--ov-menu-w").trim(),
  }));
  expect(`${widths.rendered}px`).toBe(widths.token);
  await expect(menu).not.toContainText("This dictation only");
  const rows = page.locator(
    `[data-testid="${MENU}-app"], [data-testid="${MENU}-favorite"]`,
  );
  for (const name of REAL_NAMES.filter((n) => n.length <= READABLE_LENGTH)) {
    const row = rows.filter({
      has: page.locator(".smenu-name", { hasText: name }),
    });
    await expect(row).toHaveCount(1);
    const label = row.locator(".smenu-name");
    await expect(label).toHaveText(name);
    const fit = await label.evaluate((element) => ({
      scroll: element.scrollWidth,
      client: element.clientWidth,
    }));
    expect(fit.client).toBeGreaterThan(0);
    expect(fit.scroll, `${name} is truncated`).toBeLessThanOrEqual(fit.client);
    // The star stays a visible, named toggle button with its pressed state.
    const star = row.locator("xpath=following-sibling::button[1]");
    await expect(star).toBeVisible();
    await expect(star).toHaveAttribute("aria-pressed", /^(true|false)$/);
    const starName = (await star.getAttribute("aria-label")) ?? "";
    expect(starName).toContain(name);
  }
});
