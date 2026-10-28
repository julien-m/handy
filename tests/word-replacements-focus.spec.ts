/**
 * Replacement rules editor (feature 002) in headless Chromium, against the fake Tauri IPC of
 * `tests/harness/word-replacements-fake.ts`: the keyboard focus after Done, Save, Discard and a
 * closed dialog (native session finding N5), and the refused-import alert inside the section
 * (N6). Not a native window proof.
 */
import { expect, test, type Page } from "@playwright/test";

const HARNESS = "/tests/harness/word-replacements.html";
const RULE = "replacement-settings-rule";

/** Where focus is: the focused element's test id, its tag, or `body`. */
const focused = (page: Page): Promise<string> =>
  page.evaluate(() => {
    const element = document.activeElement;
    if (element === null || element === document.body) return "body";
    return element.getAttribute("data-testid") ?? element.tagName;
  });

const open = async (page: Page): Promise<void> => {
  await page.goto(HARNESS);
  await expect(page.getByTestId(RULE)).toHaveCount(2);
};

/** Edit the first rule from the keyboard and type into its recognized field. */
const editFirstRule = async (page: Page): Promise<void> => {
  await page.getByTestId(`${RULE}-edit`).first().focus();
  await page.keyboard.press("Enter");
  const field = page.getByTestId(RULE).first().getByRole("textbox").first();
  await expect(field).toBeFocused();
  await field.fill("teh!");
};

/** The section title holds the focus (its stable point). */
const expectTitleFocused = async (page: Page): Promise<void> => {
  await expect
    .poll(() =>
      page.evaluate(() => document.activeElement?.textContent ?? "body"),
    )
    .toMatch(/replacements/i);
  expect(await focused(page)).toBe("H2");
};

test("Enter in an inline edit gives the focus back to the row's Edit button", async ({
  page,
}) => {
  await open(page);
  await editFirstRule(page);
  await page.keyboard.press("Enter");
  await expect.poll(() => focused(page)).toBe(`${RULE}-edit`);
});

test("Done clicked without focusing it (WebKit) gives the focus back to Edit", async ({
  page,
}) => {
  await open(page);
  await editFirstRule(page);
  // WebKit does not focus a clicked button: dispatch the click while the field keeps the focus.
  await page.getByTestId(`${RULE}-edit`).first().dispatchEvent("click");
  await expect.poll(() => focused(page)).toBe(`${RULE}-edit`);
});

test("Save gives the focus to the section title once the bar closes", async ({
  page,
}) => {
  await open(page);
  await editFirstRule(page);
  await page.keyboard.press("Enter");
  const save = page.getByTestId("replacement-settings-save");
  await save.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("replacement-settings-draft")).toBeHidden();
  await expectTitleFocused(page);
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});

test("Discard gives the focus to the section title", async ({ page }) => {
  await open(page);
  await editFirstRule(page);
  await page.keyboard.press("Enter");
  await page.getByTestId("replacement-settings-discard").focus();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("replacement-settings-draft")).toBeHidden();
  await expectTitleFocused(page);
});

test("the switch dialog closed by Discard gives the focus back to its trigger, the Text view switch", async ({
  page,
}) => {
  await open(page);
  await editFirstRule(page);
  await page.keyboard.press("Enter");
  const textView = page.getByTestId("replacement-settings-view-text");
  await textView.focus();
  // Marks the trigger element itself: its test id follows the screen, the element stays.
  await textView.evaluate((element) =>
    element.setAttribute("data-probe", "trigger"),
  );
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Discard", exact: true }).click();
  await expect(dialog).toBeHidden();
  await expect
    .poll(() =>
      page.evaluate(() => document.activeElement?.getAttribute("data-probe")),
    )
    .toBe("trigger");
  // The text view is now shown and the focused switch is its own.
  expect(await focused(page)).toBe("replacement-settings-bulk-view-text");
});

test("a refused import file is announced under the Replacements title, inside its section", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("replacement-settings-import").click();
  const alert = page.getByTestId("replacement-settings-import-unreadable");
  await expect(alert).toBeVisible();
  const section = page.getByTestId("replacement-settings-replacements");
  await expect(
    section.getByTestId("replacement-settings-import-unreadable"),
  ).toBeVisible();
  const title = section.getByRole("heading", { name: /replacements/i });
  const titleBox = await title.boundingBox();
  const alertBox = await alert.boundingBox();
  expect(titleBox).not.toBeNull();
  expect(alertBox!.y).toBeGreaterThan(titleBox!.y + titleBox!.height);
  // Dismiss gives the focus to the section title, not to the page.
  await page
    .getByTestId("replacement-settings-import-unreadable-dismiss")
    .focus();
  await page.keyboard.press("Enter");
  await expect(alert).toBeHidden();
  await expectTitleFocused(page);
});

test("a dialog opened while nothing had the focus returns it to the section title", async ({
  page,
}) => {
  await open(page);
  await editFirstRule(page);
  await page.keyboard.press("Enter");
  // As after a native file picker: the trigger was not focused when the dialog opened.
  await page.evaluate(() => (document.activeElement as HTMLElement).blur());
  await page
    .getByTestId("replacement-settings-view-text")
    .dispatchEvent("click");
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expectTitleFocused(page);
});
