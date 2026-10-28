/**
 * Recovery bubble (010) in headless Chromium: the real recovery window and the real output
 * settings row against the fake Tauri IPC of `tests/harness/recovery-bubble-fake.ts`. Open and
 * closed states, the first render reported once per bubble, Copy and Close with the exact
 * identity, the explanation shown and hidden, the late read updating the same bubble, the drag
 * gesture threshold, no drag handle without drag, keyboard reach of every control, a two-line
 * preview for a long right-to-left text, and the toggle (Tab, Space, announcement, absent with
 * the gate closed or off macOS). Not a native proof: first click without activation, native
 * drag and VoiceOver are native checks with Julien. `RECOVERY_BUBBLE_BASELINES=1` also saves
 * the screenshots compared with the mockups. Run with `bun run test:recovery-bubble-ui`.
 *
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: dismissible_layer, has_drag_drop, has_tooltip, is_keyboard_navigable
 */
import { expect, test, type Page } from "@playwright/test";

const BUBBLE = "/tests/harness/recovery-bubble.html";
const TOGGLE = "/tests/harness/recovery-toggle.html";
const BASELINES = ".specs/features/010-recovery-bubble/baselines";
const PREVIEW =
  "Hi Marie, the release moves to Thursday. I'll send the checklist tonight so we can review the plan together tomorrow morning before the meeting with the whole team.";

type Call = { command: string; args: Record<string, unknown> };
/**
 * Test id `suffix` under whichever contract screen the bubble shows (D1: the base screen,
 * `-explanation`, `-no-field` or `-insertion-failed`).
 */
const anyScreen = (suffix: string): RegExp =>
  new RegExp(
    `^recovery-bubble(-explanation|-no-field|-insertion-failed)?-${suffix}$`,
  );
const view = (patch: Record<string, unknown> = {}) => ({
  bubble_id: 3,
  destination_generation: 9,
  reason: "app_closed",
  app_name: "Notes",
  preview: PREVIEW,
  truncated: false,
  detail: "not_inserted",
  late_read: false,
  history: "kept",
  can_drag: true,
  ...patch,
});
const show = async (page: Page, shown: Record<string, unknown>) => {
  await page.evaluate((payload) => {
    const harness = (
      window as unknown as {
        harness: { emit: (e: string, p: unknown) => void };
      }
    ).harness;
    harness.emit("recovery-bubble-changed", payload);
  }, shown);
  await expect(page.getByTestId(anyScreen("bubble"))).toBeVisible();
};
const calls = (page: Page, command: string): Promise<Call[]> =>
  page.evaluate(
    (name) =>
      (
        window as unknown as { harness: { backend: { calls: Call[] } } }
      ).harness.backend.calls.filter((call) => call.command === name),
    command,
  );
const open = async (page: Page, shown = view()) => {
  await page.goto(BUBBLE);
  await expect(page.getByTestId("recovery-bubble-screen")).toBeAttached();
  await show(page, shown);
};
const baseline = async (page: Page, name: string) => {
  if (process.env.RECOVERY_BUBBLE_BASELINES !== "1") return;
  await page
    .getByTestId(anyScreen("bubble"))
    .screenshot({ path: `${BASELINES}/${name}.png` });
};

/** Rendered text lines of the preview text (its height over its line height). */
const previewLines = (page: Page): Promise<number> =>
  page
    .getByTestId("recovery-bubble-preview")
    .locator(".rb-preview-text")
    .evaluate((element) =>
      Math.round(
        element.clientHeight / parseFloat(getComputedStyle(element).lineHeight),
      ),
    );

// @spec AC-003: Bubble content, names and first render — .specs/features/010-recovery-bubble/spec.md#ac-003
test("open state: reason, two-line preview, actions and one first-render report", async ({
  page,
}) => {
  await open(page);
  const bubble = page.getByTestId("recovery-bubble-bubble");
  await expect(bubble).toHaveAttribute("data-state", "open");
  await expect(page.getByTestId("recovery-bubble-reason")).toHaveText(
    "Notes is no longer open",
  );
  await expect(page.getByRole("button", { name: "Close" })).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Drag into a field" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Copy" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Why?" })).toHaveAttribute(
    "aria-expanded",
    "false",
  );
  const preview = page.getByTestId("recovery-bubble-preview");
  await expect(preview).toBeVisible();
  expect(await previewLines(page)).toBe(2);
  await expect
    .poll(() => calls(page, "recovery_bubble_rendered"))
    .toHaveLength(1);
  expect((await calls(page, "recovery_bubble_rendered"))[0].args).toEqual({
    bubbleId: 3,
  });
  await expect
    .poll(async () => (await calls(page, "recovery_bubble_resize")).length)
    .toBeGreaterThan(0);
  await baseline(page, "recovery-bubble");
});

// @spec AC-005: Copy and Close with the bubble identity — .specs/features/010-recovery-bubble/spec.md#ac-005
test("Copy and Close send the exact bubble identity; hidden means closed", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("recovery-bubble-copy").click();
  expect((await calls(page, "recovery_bubble_copy"))[0].args).toEqual({
    bubbleId: 3,
    destinationGeneration: 9,
  });
  await page.getByTestId("recovery-bubble-close").click();
  expect((await calls(page, "recovery_bubble_close"))[0].args).toEqual({
    bubbleId: 3,
  });
  await page.evaluate(() =>
    (
      window as unknown as {
        harness: { emit: (e: string, p: unknown) => void };
      }
    ).harness.emit("recovery-bubble-hidden", null),
  );
  await expect(page.getByTestId("recovery-bubble-bubble")).toHaveCount(0);
});

test("the explanation shows and hides, saying History only when saved", async ({
  page,
}) => {
  await open(
    page,
    view({ reason: "pinned_app_not_running", app_name: "ChatGPT" }),
  );
  await page.getByTestId("recovery-bubble-why").click();
  // D1: explained, the bubble shows the explanation screen and every id carries its prefix.
  const text = page.getByTestId("recovery-bubble-explanation-explanation-text");
  await expect(text).toHaveText(
    "Handy didn't insert anything and sent no Enter. The transcript is kept in History. Open ChatGPT, then drag or copy the text.",
  );
  const why = page.getByTestId("recovery-bubble-explanation-why");
  await expect(why).toHaveText("Hide details");
  await expect(
    page.getByTestId("recovery-bubble-explanation-bubble"),
  ).toHaveAttribute("data-state", "explained");
  await expect(
    page.getByTestId("recovery-bubble-explanation-screen"),
  ).toBeAttached();
  await expect(page.getByTestId("recovery-bubble-bubble")).toHaveCount(0);
  await baseline(page, "recovery-bubble-explanation");
  await why.click();
  await expect(text).toHaveCount(0);
  await expect(page.getByTestId("recovery-bubble-bubble")).toBeVisible();
  await show(
    page,
    view({ bubble_id: 4, reason: "no_text_field", history: "not_kept" }),
  );
  await expect(
    page.getByTestId("recovery-bubble-no-field-bubble"),
  ).toBeVisible();
  await baseline(page, "recovery-bubble-no-field");
  await page.getByTestId("recovery-bubble-no-field-why").click();
  await expect(
    page.getByTestId("recovery-bubble-no-field-explanation-text"),
  ).toContainText("The transcript is not in History.");
});

// @spec AC-001: Late read and partial-insertion detail — .specs/features/010-recovery-bubble/spec.md#ac-001
test("a late read updates the same bubble, reported rendered only once", async ({
  page,
}) => {
  const failed = view({
    reason: "insertion_failed",
    detail: "may_have_reached",
    can_drag: false,
  });
  await open(page, failed);
  const why = page.getByTestId("recovery-bubble-insertion-failed-why");
  await why.click();
  await expect(
    page.getByTestId("recovery-bubble-insertion-failed-explanation"),
  ).toBeVisible();
  await baseline(page, "recovery-bubble-insertion-failed");
  await why.click();
  await show(page, { ...failed, late_read: true });
  // Visible in the bubble itself, without opening "Why?", and announced.
  await expect(page.getByTestId(anyScreen("explanation"))).toHaveCount(0);
  const late = page.getByTestId("recovery-bubble-insertion-failed-late-read");
  await expect(late).toBeVisible();
  await expect(late).toHaveAttribute("role", "status");
  await expect(late).toContainText("may have been pasted after all");
  await expect
    .poll(() => calls(page, "recovery_bubble_rendered"))
    .toHaveLength(1);
});

// @spec AC-004: Drag gesture starts the native drag — .specs/features/010-recovery-bubble/spec.md#ac-004
test("the drag starts past the threshold with the button held, never on a click", async ({
  page,
}) => {
  await open(page);
  const handle = page.getByTestId("recovery-bubble-drag-handle");
  await handle.click();
  expect(await calls(page, "recovery_bubble_begin_drag")).toHaveLength(0);
  const box = await handle.boundingBox();
  if (box === null) throw new Error("no handle box");
  await page.mouse.move(box.x + 10, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + 30, box.y + 12);
  await page.mouse.up();
  expect((await calls(page, "recovery_bubble_begin_drag"))[0].args).toEqual({
    bubbleId: 3,
  });
});

// @spec AC-006: No drag handle on Windows and Linux — .specs/features/010-recovery-bubble/spec.md#ac-006
test("without drag the handle is absent and Copy, Why? and Close remain", async ({
  page,
}) => {
  await open(page, view({ can_drag: false }));
  await expect(page.getByTestId("recovery-bubble-drag-handle")).toHaveCount(0);
  await expect(page.getByTestId("recovery-bubble-copy")).toBeVisible();
  await expect(page.getByTestId("recovery-bubble-close")).toBeVisible();
});

test("every control is reachable by Tab and Space activates Copy", async ({
  page,
}) => {
  await open(page);
  await page.getByTestId("harness-start").focus();
  const order = [
    "recovery-bubble-close",
    "recovery-bubble-drag-handle",
    "recovery-bubble-copy",
    "recovery-bubble-why",
  ];
  for (const id of order) {
    await page.keyboard.press("Tab");
    await expect(page.getByTestId(id)).toBeFocused();
  }
  await page.getByTestId("recovery-bubble-copy").focus();
  await page.keyboard.press("Space");
  await expect.poll(() => calls(page, "recovery_bubble_copy")).toHaveLength(1);
});

test("a long right-to-left text stays two lines in the preview", async ({
  page,
}) => {
  const arabic =
    "هذا نص طويل جدا من الإملاء يجب ألا يتجاوز سطرين في المعاينة ".repeat(6);
  await open(page, view({ preview: arabic, truncated: true }));
  const preview = page.getByTestId("recovery-bubble-preview");
  await expect(preview).toHaveAttribute("data-truncated", "true");
  expect(await previewLines(page)).toBe(2);
});

// @spec AC-007: Setting row, keyboard and platforms — .specs/features/010-recovery-bubble/spec.md#ac-007
test("the setting toggles with Tab and Space and is announced", async ({
  page,
}) => {
  await page.goto(TOGGLE);
  const toggle = page.getByTestId("output-settings-no-field-bubble-toggle");
  await expect(
    page.getByTestId("output-settings-no-field-bubble-title"),
  ).toHaveText("Show the bubble when no text field is focused");
  await expect(toggle).toBeChecked();
  if (process.env.RECOVERY_BUBBLE_BASELINES === "1") {
    await page
      .getByTestId("output-settings-no-field-bubble")
      .screenshot({ path: `${BASELINES}/output-settings-no-field-bubble.png` });
  }
  await page.getByTestId("harness-start").focus();
  await page.keyboard.press("Tab");
  await expect(toggle).toBeFocused();
  await expect(toggle).toHaveAccessibleName(
    "Show the bubble when no text field is focused",
  );
  await page.keyboard.press("Space");
  await expect(toggle).not.toBeChecked();
  expect(
    (
      await calls(
        page,
        "change_show_recovery_bubble_without_text_field_setting",
      )
    )[0].args,
  ).toEqual({
    enabled: false,
  });
});

test("the setting row is absent with the gate closed and off macOS", async ({
  page,
}) => {
  for (const query of ["?gate=closed", "?os=windows", "?os=linux"]) {
    await page.goto(`${TOGGLE}${query}`);
    await expect(page.getByTestId("harness-start")).toBeVisible();
    await expect(
      page.getByTestId("output-settings-no-field-bubble"),
    ).toHaveCount(0);
  }
});
