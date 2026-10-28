// Keyboard focus and pointer reorder checks on the harness, printed as one line per assertion.
import { chromium } from "playwright";
const dir = process.argv[2];
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 520, height: 260 }, deviceScaleFactor: 2, colorScheme: "dark" });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
await page.goto(`file://${dir}/harness.html`);
await page.waitForSelector('[data-testid="microphone-settings-choice"]');
const order = () => page.evaluate(() => window.currentOrder.join(" | "));
const focused = () => page.evaluate(() => {
  const el = document.activeElement;
  return `${el?.getAttribute("data-testid")} of ${el?.closest("li")?.getAttribute("data-name")}`;
});
const check = (label, actual, expected) => console.log(`${actual === expected ? "PASS" : "FAIL"} ${label}: ${actual}`);

// Keyboard: Enter on Move up of the second row.
const upOfSecond = page.locator('li[data-name="MacBook Pro Microphone"] [data-testid="microphone-settings-choice-move-up"]');
await upOfSecond.focus();
await page.keyboard.press("Enter");
check("Enter on Move up of row 2 reorders", await order(), "MacBook Pro Microphone | USB headset | Studio Display Microphone");
check("focus stays on the moved row's Move up", await focused(), "microphone-settings-choice-move-up of MacBook Pro Microphone");
// Keyboard: Move down of the first row (React moves this row's node).
const downOfFirst = page.locator('li[data-name="MacBook Pro Microphone"] [data-testid="microphone-settings-choice-move-down"]');
await downOfFirst.focus();
await page.keyboard.press("Enter");
check("Enter on Move down of row 1 reorders", await order(), "USB headset | MacBook Pro Microphone | Studio Display Microphone");
check("focus stays on the moved row's Move down", await focused(), "microphone-settings-choice-move-down of MacBook Pro Microphone");
// Edge control is inert: Move up of the first row does nothing.
await page.locator('li[data-name="USB headset"] [data-testid="microphone-settings-choice-move-up"]').click({ force: true });
check("Move up of the first row is inert", await order(), "USB headset | MacBook Pro Microphone | Studio Display Microphone");
// Tab order: the controls receive focus in reading order.
await page.locator('li[data-name="USB headset"] [data-testid="microphone-settings-choice-move-up"]').focus();
const tabbed = [];
for (let i = 0; i < 4; i += 1) { await page.keyboard.press("Tab"); tabbed.push(await focused()); }
check("Tab reading order", tabbed.join(", "), "microphone-settings-choice-move-down of USB headset, microphone-settings-choice-remove of USB headset, microphone-settings-choice-move-up of MacBook Pro Microphone, microphone-settings-choice-move-down of MacBook Pro Microphone");
// Pointer: drag the first row's handle below the third row.
const handle = page.locator('li[data-name="USB headset"] span[aria-hidden="true"]').first();
const box = await handle.boundingBox();
const target = await page.locator('li[data-name="Studio Display Microphone"]').boundingBox();
await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
await page.mouse.down();
await page.mouse.move(box.x + box.width / 2, target.y + target.height / 2, { steps: 6 });
check("list state while over a new row", await page.getAttribute('[data-testid="microphone-settings-choices"]', "data-state"), "drop-target");
check("dragged row state", await page.getAttribute('li[data-name="USB headset"]', "data-state"), "dragging");
check("target row state", await page.getAttribute('li[data-name="Studio Display Microphone"]', "data-state"), "drop-target");
await page.screenshot({ path: `${dir}/drag-drop-target.png` });
await page.mouse.up();
check("drop reorders the draft", await order(), "MacBook Pro Microphone | Studio Display Microphone | USB headset");
check("states cleared after the drop", String(await page.getAttribute('[data-testid="microphone-settings-choices"]', "data-state")), "null");
// Escape cancels a drag.
const handle2 = page.locator('li[data-name="MacBook Pro Microphone"] span[aria-hidden="true"]').first();
const b2 = await handle2.boundingBox();
await page.mouse.move(b2.x + 5, b2.y + 5); await page.mouse.down();
await page.mouse.move(b2.x + 5, b2.y + 90, { steps: 4 });
await page.keyboard.press("Escape");
await page.mouse.up();
check("Escape cancels the drag", await order(), "MacBook Pro Microphone | Studio Display Microphone | USB headset");
// Remove.
await page.locator('li[data-name="USB headset"] [data-testid="microphone-settings-choice-remove"]').click();
check("Remove drops the row from the draft", await order(), "MacBook Pro Microphone | Studio Display Microphone");
console.log(`page errors: ${errors.length ? errors.join(" / ") : "none"}`);
await browser.close();
