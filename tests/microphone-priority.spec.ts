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
  anyScreen,
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
    // The write in flight keeps the bar's control Saving… until it settles.
    await expect(save).toHaveAttribute(
      "data-testid",
      "microphone-settings-error-save",
    );
    // The held write was sent at the old revision: it is refused, nothing is replaced, and the
    // bar now offers Reload in place of Save (native session N3).
    await page.evaluate(() => window.harness.releaseSave(null));
    await expect(reload).not.toHaveAttribute("aria-disabled", "true");
    await expect(
      page.getByTestId("microphone-settings-error-draft-reload"),
    ).toBeVisible();
    await expect(page.getByTestId(anyScreen("save"))).toHaveCount(0);
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-error-conflict"),
    ).toBeHidden();
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual([STUDIO]);
  });

  test("after a change saved elsewhere, the bar offers Reload instead of Save until Reload or Discard", async ({
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
    // The bar has no Save during the conflict: Discard and Reload only (native session N3).
    const bar = page.getByTestId("microphone-settings-error-draft");
    await expect(bar.getByRole("button")).toHaveText([
      text.draft.discard,
      text.conflict.reload,
    ]);
    await expect(page.getByTestId(anyScreen("save"))).toHaveCount(0);
    // Another edit keeps the conflict, and nothing is written.
    await control(page, USB, "move-up").focus();
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-error-draft-reload"),
    ).toBeVisible();
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

  test("Reload in the bar rereads the order and moves focus to the section title", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.evaluate(() => window.harness.commitElsewhere(["Dock"]));
    const reload = page.getByTestId("microphone-settings-error-draft-reload");
    await reload.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByTestId(anyScreen("draft"))).toHaveCount(0);
    await expectFocus(page, TITLE);
    expect(await order(page)).toEqual(["Dock"]);
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

test.describe("compact save bar (native session N2)", () => {
  test("hidden without a change, then kept at the bottom of the scrolled settings content", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 900, height: 300 });
    await open(page, "?inWindow");
    await expect(page.getByTestId(anyScreen("draft"))).toHaveCount(0);
    await editFromKeyboard(page);
    const bar = page.getByTestId("microphone-settings-draft");
    const content = page.getByTestId("settings-window-content");
    // The editor is taller than the content area: without sticking, the bar would be out of view.
    const overflow = await content.evaluate(
      (element) => element.scrollHeight - element.clientHeight,
    );
    expect(overflow).toBeGreaterThan(100);
    await expect(bar.getByRole("button")).toHaveText([
      text.draft.discard,
      text.draft.save,
    ]);
    const barSitsAtContentBottom = async (): Promise<void> => {
      const area = await content.boundingBox();
      const box = await bar.boundingBox();
      expect(area).not.toBeNull();
      expect(box).not.toBeNull();
      const areaBottom = area!.y + area!.height;
      const barBottom = box!.y + box!.height;
      expect(barBottom).toBeLessThanOrEqual(areaBottom);
      expect(barBottom).toBeGreaterThan(areaBottom - 40);
    };
    // Content scrolled to the top, then halfway: the bar stays at the bottom of the visible
    // content, and only the content area scrolls, never the window.
    for (const position of [0, Math.floor(overflow / 2)]) {
      await content.evaluate((element, top) => {
        element.scrollTop = top;
      }, position);
      await expect
        .poll(() => content.evaluate((element) => element.scrollTop))
        .toBe(position);
      await barSitsAtContentBottom();
    }
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
  });
});
