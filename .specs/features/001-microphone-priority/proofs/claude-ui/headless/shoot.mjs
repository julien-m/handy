import { chromium } from "playwright";
const dir = process.argv[2];
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 547, height: 600 }, deviceScaleFactor: 2, colorScheme: "dark" });
for (const name of ["microphone-settings", "microphone-settings-empty", "microphone-settings-error"]) {
  await page.goto(`file://${dir}/${name}.html`);
  await page.screenshot({ path: `${dir}/${name}.png`, fullPage: true });
}
await browser.close();
