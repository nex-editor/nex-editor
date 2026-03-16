import { expect, test } from "@playwright/test";

const paddingX = 24;
const lineY = 30;
const charWidth = 9.6;

test("canvas editor accepts typing and deletion", async ({ page }) => {
  await page.goto("/");

  const status = page.locator("#status");
  const revision = page.locator("#revision");
  const canvas = page.locator("#editor");

  await expect(status).toContainText("text length: 0", { timeout: 15_000 });

  await canvas.click();
  await page.keyboard.type("hello");

  await expect(status).toContainText("text length: 5");
  await expect(revision).toContainText("revision: 5");

  await page.keyboard.press("Backspace");
  await expect(status).toContainText("text length: 4");

  await page.keyboard.press(`${process.platform === "darwin" ? "Meta" : "Control"}+A`);
  await expect(status).toContainText("selection: 0 → 4");

  await page.keyboard.type("X");
  await expect(status).toContainText("text length: 1");
  await expect(status).toContainText("selection: 1 → 1");
});

test("canvas editor supports multiline vertical movement", async ({ page }) => {
  await page.goto("/");

  const status = page.locator("#status");
  const canvas = page.locator("#editor");

  await expect(status).toContainText("text length: 0", { timeout: 15_000 });

  await canvas.click();
  await page.keyboard.type("abc");
  await page.keyboard.press("Enter");
  await page.keyboard.type("def");

  await expect(status).toContainText("selection: 7 → 7");

  await page.keyboard.press("ArrowUp");
  await expect(status).toContainText("selection: 3 → 3");

  await page.keyboard.press("ArrowDown");
  await expect(status).toContainText("selection: 7 → 7");
});

test("canvas editor supports pointer placement and drag selection", async ({ page }) => {
  await page.goto("/");

  const status = page.locator("#status");
  const canvas = page.locator("#editor");

  await expect(status).toContainText("text length: 0", { timeout: 15_000 });

  await canvas.click();
  await page.keyboard.type("hello");
  await expect(status).toContainText('text: "hello"');

  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("Canvas bounding box unavailable");
  }

  await page.mouse.click(
    box.x + paddingX + charWidth * 1.2,
    box.y + lineY,
  );
  await expect(status).toContainText("selection: 1 → 1");

  await page.mouse.move(
    box.x + paddingX + charWidth * 0.2,
    box.y + lineY,
  );
  await page.mouse.down();
  await page.mouse.move(
    box.x + paddingX + charWidth * 4.2,
    box.y + lineY,
    { steps: 5 },
  );
  await page.mouse.up();

  await expect(status).toContainText("selection: 0 → 4");

  await page.keyboard.type("X");
  await expect(status).toContainText("text length: 2");
  await expect(status).toContainText('text: "Xo"');
  await expect(status).toContainText("selection: 1 → 1");
});
