/**
 * Microphone priority editor (feature 001) in headless Chromium: the real React editor, settings
 * store and status hook against the fake Tauri IPC of `tests/harness/`. Covers keyboard focus
 * after every action, including a write whose outcome is unknown and an order saved elsewhere
 * (SC-003), the loading guard and a failed device listing (FR-002). Status polling and pointer
 * drags have their own specs (`microphone-priority-polling.spec.ts`,
 * `microphone-priority-drag.spec.ts`). Not a native window proof. Run with
 * `bun run test:microphone-priority-ui`.
 */
import { expect, test, type Page } from "@playwright/test";
import {
  MACBOOK,
  ORIGINAL,
  STUDIO,
  USB,
  control,
  editFromKeyboard,
  expectFocus,
  open,
  order,
  row,
  startHeldSave,
  text,
} from "./harness/microphone-priority-page";

const TITLE = "microphone-settings-priority-title";

/** Save from the keyboard, held then refused: Save keeps focus while busy and as Retry. */
const saveThenFail = async (page: Page): Promise<void> => {
  const save = await startHeldSave(page);
  await page.keyboard.press("Enter");
  const calls = () => page.evaluate(() => window.harness.backend.saveCalls);
  expect((await calls()).length).toBe(1);
  await page.evaluate(() => window.harness.releaseSave("persistence_failed"));
  await expect(save).toHaveText(text.retry);
  const screen = page.getByTestId("microphone-settings-screen");
  await expect(screen).toHaveAttribute("data-state", "error");
  await expectFocus(page, "microphone-settings-save");
};

test.describe("focus after each action (SC-003)", () => {
  test("Save keeps focus while busy and when it turns into Retry", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await saveThenFail(page);
  });

  test("a successful Retry marks the editor saved and moves focus to the title", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await saveThenFail(page);
    await page.evaluate(() => window.harness.releaseSave(null));
    await page.keyboard.press("Enter");
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    const screen = page.getByTestId("microphone-settings-screen");
    await expect(screen).toHaveAttribute("data-state", "success");
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
  });

  test("Remove moves focus to the next row, else the previous one, else Add", async ({
    page,
  }) => {
    await open(page);
    await control(page, USB, "remove").focus();
    await page.keyboard.press("Enter");
    await expectFocus(page, `microphone-settings-choice-remove of ${MACBOOK}`);
    await control(page, STUDIO, "remove").focus();
    await page.keyboard.press("Enter");
    await expectFocus(page, `microphone-settings-choice-remove of ${MACBOOK}`);
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-empty-empty"),
    ).toBeVisible();
    await expect(
      page.getByTestId("microphone-settings-empty-screen"),
    ).toBeVisible();
    await expectFocus(page, "microphone-settings-empty-add (combobox)");
  });

  test("a keyboard move keeps focus on the moved row's control", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await expectFocus(page, `microphone-settings-choice-move-up of ${MACBOOK}`);
    await control(page, MACBOOK, "move-down").focus();
    await page.keyboard.press("Enter");
    await expectFocus(
      page,
      `microphone-settings-choice-move-down of ${MACBOOK}`,
    );
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
  });

  test("Discard moves focus to the section title", async ({ page }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.getByTestId("microphone-settings-discard").focus();
    await page.keyboard.press("Enter");
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual(ORIGINAL);
  });

  test("Reload moves focus to the section title", async ({ page }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.evaluate(() => window.harness.commitElsewhere(["Dock"]));
    const reload = page.getByTestId("microphone-settings-error-reload");
    await expect(reload).toBeVisible();
    await reload.focus();
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-error-conflict"),
    ).toBeHidden();
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual(["Dock"]);
  });

  test("Reload is inactive while a Save is in flight", async ({ page }) => {
    await open(page);
    await editFromKeyboard(page);
    const save = await startHeldSave(page);
    // Saved elsewhere while the write is held: the banner shows, Reload is inactive meanwhile.
    await page.evaluate(
      (name) => window.harness.commitElsewhere([name]),
      STUDIO,
    );
    const reload = page.getByTestId("microphone-settings-error-reload");
    await expect(reload).toHaveAttribute("aria-disabled", "true");
    // Still focusable (aria-disabled), but activating it does nothing during the write.
    await reload.focus();
    await page.keyboard.press("Enter");
    await expectFocus(page, "microphone-settings-error-reload");
    await expect(
      page.getByTestId("microphone-settings-error-draft"),
    ).toBeVisible();
    await expect(save).toHaveAttribute(
      "data-testid",
      "microphone-settings-error-save",
    );
    // The held write was sent at the old revision: it is refused, nothing is replaced.
    await page.evaluate(() => window.harness.releaseSave(null));
    await expect(reload).not.toHaveAttribute("aria-disabled", "true");
    await expect(save).toHaveAttribute("aria-disabled", "true");
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-error-conflict"),
    ).toBeHidden();
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual([STUDIO]);
  });

  test("after a change saved elsewhere, Save stays inactive until Reload or Discard", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.evaluate(() => window.harness.commitElsewhere(["Dock"]));
    const conflict = page.getByTestId("microphone-settings-error-conflict");
    await expect(conflict).toBeVisible();
    // The banner says the truth: Reload shows the other order and discards the edits.
    await expect(conflict).toContainText(text.conflict.body);
    // Every test id of the shown screen carries the error prefix, so none is duplicated.
    await expect(
      page.getByTestId("microphone-settings-error-screen"),
    ).toBeVisible();
    await expect(page.getByTestId("microphone-settings-screen")).toHaveCount(0);
    const save = page.getByTestId("microphone-settings-error-save");
    await expect(save).toHaveAttribute("aria-disabled", "true");
    await expect(save).toHaveAttribute("data-state", "disabled");
    // Another edit keeps it inactive, and activating it writes nothing.
    await control(page, USB, "move-up").focus();
    await page.keyboard.press("Enter");
    await expect(save).toHaveAttribute("aria-disabled", "true");
    await save.focus();
    await page.keyboard.press("Enter");
    await save.click({ force: true });
    const calls = () => page.evaluate(() => window.harness.backend.saveCalls);
    expect((await calls()).length).toBe(0);
    // Discard ends the conflict: the order saved elsewhere is shown, nothing was written.
    await page.getByTestId("microphone-settings-error-discard").click();
    await expect(
      page.getByTestId("microphone-settings-error-conflict"),
    ).toBeHidden();
    expect(await order(page)).toEqual(["Dock"]);
    expect((await calls()).length).toBe(0);
  });

  test("an unknown outcome that did commit is confirmed and focus moves to the title", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await startHeldSave(page);
    await page.evaluate(() => window.harness.releaseSaveUnknown(true));
    await expect(
      page.getByTestId("microphone-settings-screen"),
    ).toHaveAttribute("data-state", "success");
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
  });

  test("an unknown outcome that did not commit keeps the draft and Retry with focus", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    const save = await startHeldSave(page);
    await page.evaluate(() => window.harness.releaseSaveUnknown(false));
    await expect(save).toHaveText(text.retry);
    await expect(
      page.getByTestId("microphone-settings-screen"),
    ).toHaveAttribute("data-state", "error");
    await expectFocus(page, "microphone-settings-save");
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
  });

  test("the same order saved elsewhere while Save has focus: focus moves to the title", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.getByTestId("microphone-settings-save").focus();
    await page.evaluate(
      (draft) => window.harness.commitElsewhere(draft),
      [MACBOOK, USB, STUDIO],
    );
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    await expectFocus(page, TITLE);
    await expect(
      page.getByTestId("microphone-settings-screen"),
    ).not.toHaveAttribute("data-state", /.+/);
  });

  test("the same order saved elsewhere leaves a focus that is still on a row alone", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.evaluate(
      (draft) => window.harness.commitElsewhere(draft),
      [MACBOOK, USB, STUDIO],
    );
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
    await expectFocus(page, `microphone-settings-choice-move-up of ${MACBOOK}`);
  });
});

test.describe("loading guard and failed listing", () => {
  test("Add and Save are inactive until the settings are loaded", async ({
    page,
  }) => {
    await open(page, "?holdSettings");
    await expect(page.getByTestId("microphone-settings-loading")).toBeVisible();
    const add = page
      .getByTestId("microphone-settings-add")
      .getByRole("combobox");
    await expect(add).toBeDisabled();
    await page.evaluate(() => window.harness.releaseSettings());
    await expect(page.getByTestId("microphone-settings-choices")).toBeVisible();
    await expect(add).toBeEnabled();
  });

  test("a failed listing: Unknown for a draft-only name, Add inactive until a Refresh lists again (FR-002)", async ({
    page,
  }) => {
    await open(page);
    const add = page
      .getByTestId("microphone-settings-add")
      .getByRole("combobox");
    await add.click();
    await page.getByRole("option", { name: "Dock" }).click();
    const dock = row(page, "Dock").getByTestId(
      "microphone-settings-choice-status",
    );
    await expect(dock).toHaveText(text.status.available);
    await expect(page.getByTestId("microphone-settings-add")).toHaveAttribute(
      "data-state",
      "valid",
    );
    await page.evaluate(() => {
      window.harness.backend.listingFails = true;
    });
    const refresh = page.getByTestId("microphone-settings-refresh");
    await refresh.click();
    await expect(dock).toHaveText(text.status.unknown);
    await expect(
      row(page, USB).getByTestId("microphone-settings-choice-status"),
    ).toHaveText(text.status.available);
    await expect(add).toBeDisabled();
    await expect(add).toHaveAccessibleName(text.refreshFailed);
    await page.evaluate(() => {
      window.harness.backend.listingFails = false;
    });
    await refresh.click();
    await expect(add).toBeEnabled();
    await expect(add).toHaveAccessibleName(text.addLabel);
  });
});
