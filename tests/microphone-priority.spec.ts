/**
 * Microphone priority editor (feature 001) in headless Chromium: the real React editor, settings
 * store and status hook against the fake Tauri IPC of `tests/harness/`. Covers keyboard focus
 * after every action (SC-003), status polling on a simulated clock with page visibility (AC-002),
 * interrupted pointer drags, the loading guard and a failed device listing (FR-002).
 * Not a native window proof. Run with
 * `bun run test:playwright -- tests/microphone-priority.spec.ts --reporter=line`.
 */
import { expect, test, type Locator, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";

interface Harness {
  backend: {
    statusCalls: number;
    saveCalls: unknown[];
    listingFails: boolean;
  };
  commitElsewhere: (order: string[]) => void;
  holdSave: () => void;
  releaseSave: (failure: string | null) => void;
  releaseSettings: () => void;
  setVisible: (visible: boolean) => void;
  unmount: () => void;
}

declare global {
  interface Window {
    harness: Harness;
  }
}

const en = JSON.parse(
  readFileSync(
    new URL("../src/i18n/locales/en/translation.json", import.meta.url),
    "utf8",
  ),
);
const text = en.settings.sound.microphonePriority;
const HARNESS = "/tests/harness/microphone-priority.html";
const USB = "USB headset";
const MACBOOK = "MacBook Pro Microphone";
const STUDIO = "Studio Display Microphone";
const ORIGINAL = [USB, MACBOOK, STUDIO];

const open = async (page: Page, query = ""): Promise<void> => {
  await page.goto(`${HARNESS}${query}`);
  await expect(page.getByTestId("microphone-settings-screen")).toBeVisible();
};

const row = (page: Page, name: string): Locator =>
  page.locator(`li[data-name="${name}"]`);

const control = (page: Page, name: string, id: string): Locator =>
  row(page, name).getByTestId(`microphone-settings-choice-${id}`);

const order = (page: Page): Promise<string[]> =>
  page
    .getByTestId("microphone-settings-choice")
    .evaluateAll((rows) =>
      rows.map((item) => item.getAttribute("data-name") ?? ""),
    );

/** Where focus is: the focused element's test ID (or role) and the row that holds it. */
const focused = (page: Page) =>
  page.evaluate(() => {
    const element = document.activeElement;
    if (element === null || element === document.body) return "body";
    const id =
      element.getAttribute("data-testid") ??
      element.closest("[data-testid]")?.getAttribute("data-testid");
    const name = element.closest("li")?.getAttribute("data-name");
    const role = element.getAttribute("role");
    if (name) return `${id} of ${name}`;
    return role ? `${id} (${role})` : `${id}`;
  });

/** Make the draft dirty from the keyboard: Move up on the second row. */
const editFromKeyboard = async (page: Page): Promise<void> => {
  await control(page, MACBOOK, "move-up").focus();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("microphone-settings-draft")).toBeVisible();
};

/** Save from the keyboard, held then refused: Save keeps focus while busy and as Retry. */
const saveThenFail = async (page: Page): Promise<void> => {
  const save = page.getByTestId("microphone-settings-save");
  await page.evaluate(() => window.harness.holdSave());
  await save.focus();
  await page.keyboard.press("Enter");
  await expect(save).toHaveAttribute("data-state", "loading");
  await expect(save).toHaveAttribute("aria-disabled", "true");
  expect(await focused(page)).toBe("microphone-settings-save");
  await page.keyboard.press("Enter");
  const calls = () => page.evaluate(() => window.harness.backend.saveCalls);
  expect((await calls()).length).toBe(1);
  await page.evaluate(() => window.harness.releaseSave("persistence_failed"));
  await expect(save).toHaveText(text.retry);
  const screen = page.getByTestId("microphone-settings-screen");
  await expect(screen).toHaveAttribute("data-state", "error");
  expect(await focused(page)).toBe("microphone-settings-save");
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
    expect(await focused(page)).toBe("microphone-settings-priority-title");
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
  });

  test("Remove moves focus to the next row, else the previous one, else Add", async ({
    page,
  }) => {
    await open(page);
    await control(page, USB, "remove").focus();
    await page.keyboard.press("Enter");
    expect(await focused(page)).toBe(
      `microphone-settings-choice-remove of ${MACBOOK}`,
    );
    await control(page, STUDIO, "remove").focus();
    await page.keyboard.press("Enter");
    expect(await focused(page)).toBe(
      `microphone-settings-choice-remove of ${MACBOOK}`,
    );
    await page.keyboard.press("Enter");
    await expect(
      page.getByTestId("microphone-settings-empty-empty"),
    ).toBeVisible();
    expect(await focused(page)).toBe("microphone-settings-add (combobox)");
  });

  test("a keyboard move keeps focus on the moved row's control", async ({
    page,
  }) => {
    await open(page);
    await editFromKeyboard(page);
    expect(await focused(page)).toBe(
      `microphone-settings-choice-move-up of ${MACBOOK}`,
    );
    await control(page, MACBOOK, "move-down").focus();
    await page.keyboard.press("Enter");
    expect(await focused(page)).toBe(
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
    expect(await focused(page)).toBe("microphone-settings-priority-title");
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
    expect(await focused(page)).toBe("microphone-settings-priority-title");
    expect(await order(page)).toEqual(["Dock"]);
  });

  test("Reload is inactive while a Save is in flight", async ({ page }) => {
    await open(page);
    await editFromKeyboard(page);
    await page.evaluate(
      (name) => window.harness.commitElsewhere([name]),
      STUDIO,
    );
    const reload = page.getByTestId("microphone-settings-error-reload");
    await expect(reload).toBeVisible();
    await page.evaluate(() => window.harness.holdSave());
    await page.getByTestId("microphone-settings-save").click();
    await expect(reload).toHaveAttribute("aria-disabled", "true");
    // Still focusable (aria-disabled), but activating it does nothing during the write.
    await reload.focus();
    await page.keyboard.press("Enter");
    expect(await focused(page)).toBe("microphone-settings-error-reload");
    await expect(page.getByTestId("microphone-settings-draft")).toBeVisible();
    await page.evaluate(() => window.harness.releaseSave(null));
    await expect(
      page.getByTestId("microphone-settings-screen"),
    ).toHaveAttribute("data-state", "success");
    await expect(
      page.getByTestId("microphone-settings-error-conflict"),
    ).toBeHidden();
    expect(await focused(page)).toBe("microphone-settings-priority-title");
    expect(await order(page)).toEqual([MACBOOK, USB, STUDIO]);
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

  test("a draft-only name shows Unknown when the device listing fails (FR-002)", async ({
    page,
  }) => {
    await open(page);
    await page
      .getByTestId("microphone-settings-add")
      .getByRole("combobox")
      .click();
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
    await page.getByTestId("microphone-settings-refresh").click();
    await expect(dock).toHaveText(text.status.unknown);
    await expect(
      row(page, USB).getByTestId("microphone-settings-choice-status"),
    ).toHaveText(text.status.available);
  });
});

test("status polling: on open, every 5 s while visible, paused while hidden, cleared on unmount (AC-002)", async ({
  page,
}) => {
  await page.clock.install();
  await open(page);
  const reads = () => page.evaluate(() => window.harness.backend.statusCalls);
  // Read on open.
  await expect.poll(reads).toBe(1);
  // From here the page clock only moves with runFor.
  await page.clock.pauseAt(new Date(Date.now() + 2000));
  await page.evaluate(() => window.harness.setVisible(false));
  const base = await reads();
  await page.clock.runFor(60_000);
  expect(await reads()).toBe(base);
  // Shown again: a read at once, then one every 5 s.
  await page.evaluate(() => window.harness.setVisible(true));
  await expect.poll(reads).toBe(base + 1);
  await page.clock.runFor(4999);
  expect(await reads()).toBe(base + 1);
  await page.clock.runFor(1);
  await expect.poll(reads).toBe(base + 2);
  await page.clock.runFor(5000);
  await expect.poll(reads).toBe(base + 3);
  // Unmounted: no more reads.
  await page.evaluate(() => window.harness.unmount());
  await page.clock.runFor(30_000);
  expect(await reads()).toBe(base + 3);
});

test.describe("interrupted pointer drag", () => {
  const startDrag = async (page: Page): Promise<void> => {
    await page.evaluate(() =>
      document.addEventListener(
        "pointerdown",
        (event) => {
          document.body.dataset.pointerId = String(event.pointerId);
        },
        true,
      ),
    );
    const handle = row(page, USB).locator('span[aria-hidden="true"]').first();
    const from = await handle.boundingBox();
    const to = await row(page, STUDIO).boundingBox();
    if (from === null || to === null) throw new Error("rows not laid out");
    await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
    await page.mouse.down();
    await page.mouse.move(from.x + from.width / 2, to.y + to.height / 2, {
      steps: 6,
    });
    await expect(
      page.getByTestId("microphone-settings-choices"),
    ).toHaveAttribute("data-state", "drop-target");
    await expect(row(page, USB)).toHaveAttribute("data-state", "dragging");
  };

  const expectCancelled = async (page: Page): Promise<void> => {
    const list = page.getByTestId("microphone-settings-choices");
    await expect.poll(() => list.getAttribute("data-state")).toBeNull();
    await expect
      .poll(() => row(page, USB).getAttribute("data-state"))
      .toBeNull();
    await page.mouse.up();
    expect(await order(page)).toEqual(ORIGINAL);
    await expect(page.getByTestId("microphone-settings-draft")).toBeHidden();
  };

  const interruptions: Record<string, (page: Page) => Promise<unknown>> = {
    "window blur": (page) =>
      page.evaluate(() => window.dispatchEvent(new Event("blur"))),
    "page hidden": (page) =>
      page.evaluate(() => window.harness.setVisible(false)),
    "lost pointer capture": (page) =>
      page.evaluate(() => {
        const handle = document.querySelector<HTMLElement>(
          'li[data-name="USB headset"] span[aria-hidden="true"]',
        );
        handle?.releasePointerCapture(Number(document.body.dataset.pointerId));
      }),
    "pointer cancel": (page) =>
      page.evaluate(() => {
        const handle = document.querySelector<HTMLElement>(
          'li[data-name="USB headset"] span[aria-hidden="true"]',
        );
        const pointerId = Number(document.body.dataset.pointerId);
        handle?.dispatchEvent(
          new PointerEvent("pointercancel", { bubbles: true, pointerId }),
        );
      }),
  };

  for (const [name, interrupt] of Object.entries(interruptions)) {
    test(`${name} cancels the drag without reordering`, async ({ page }) => {
      await open(page);
      await startDrag(page);
      await interrupt(page);
      await expectCancelled(page);
    });
  }

  test("an uninterrupted drag still drops", async ({ page }) => {
    await open(page);
    await startDrag(page);
    await page.mouse.up();
    expect(await order(page)).toEqual([MACBOOK, STUDIO, USB]);
    await expect(page.getByTestId("microphone-settings-draft")).toBeVisible();
  });
});
