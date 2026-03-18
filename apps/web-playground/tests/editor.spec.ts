import { expect, test } from "@playwright/test";

import { createPlaygroundHarness } from "./hooks";

test.beforeEach(async ({ page }) => {
  const playground = createPlaygroundHarness(page);
  await playground.open();
  await playground.expectReady();
});

test("canvas editor accepts typing and deletion", async ({ page }) => {
  const playground = createPlaygroundHarness(page);

  await expect(playground.docJson).toContainText('"block_type": "doc"');

  await playground.focusEditor();
  await page.keyboard.type("hello");

  await expect(playground.status).toContainText("text length: 5");
  await expect(playground.revision).toContainText("revision: 5");
  await expect(playground.docJson).toContainText('"text": "hello"');
  await expect(playground.operationLog).toContainText(
    'InsertText {"text":"o"}',
  );
  await expect(playground.operationLog).toContainText("revision=5");

  await page.keyboard.press("Backspace");
  await expect(playground.status).toContainText("text length: 4");
  await expect(playground.operationLog).toContainText("Backspace");

  await page.keyboard.press(
    `${process.platform === "darwin" ? "Meta" : "Control"}+A`,
  );
  await expect(playground.status).toContainText("selection: 0 → 4");

  await page.keyboard.type("X");
  await expect(playground.status).toContainText("text length: 1");
  await expect(playground.status).toContainText("selection: 1 → 1");
  await expect(playground.docJson).toContainText('"text": "X"');
});

test("render snapshot exposes style table and style ids", async ({ page }) => {
  const playground = createPlaygroundHarness(page);
  const snapshot = await playground.snapshot<{
    scene?: {
      styles?: Array<{ id?: string; role?: string; measurement_style_key?: string | null }>;
      background?: Array<{ style_id?: string }>;
    };
  }>();

  expect(snapshot?.scene?.styles).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: "surface",
        role: "EditorSurface",
        measurement_style_key: null,
      }),
      expect.objectContaining({
        id: "text.primary",
        role: "PrimaryText",
        measurement_style_key: "text.primary",
      }),
    ]),
  );
  expect(snapshot?.scene?.background?.[0]?.style_id).toBe("surface");
});

test("canvas editor exposes composition preview before commit", async ({
  page,
}) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await playground.compositionStart();
  await playground.compositionUpdate("你");

  await expect(playground.status).toContainText('composition: "你"');
  await expect(playground.status).toContainText('text: ""');
  await expect(playground.status).toContainText('display: "你"');

  const snapshot = await playground.snapshot<{
    scene?: { composition_underlines?: unknown[] };
  }>();
  expect(snapshot?.scene?.composition_underlines?.length).toBe(1);

  await playground.compositionEnd("你");
  await expect(playground.status).toContainText("composition: idle");
  await expect(playground.status).toContainText('text: "你"');
});

test("canvas editor supports committed Chinese text input and deletion", async ({
  page,
}) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await playground.compositionStart();
  await playground.compositionUpdate("你好");
  await playground.compositionEnd("你好");

  await expect(playground.status).toContainText('text: "你好"');
  await expect(playground.status).toContainText("selection: 2 → 2");

  await page.keyboard.press("ArrowLeft");
  await expect(playground.status).toContainText("selection: 1 → 1");

  await page.keyboard.press("Backspace");
  await expect(playground.status).toContainText('text: "好"');
  await expect(playground.status).toContainText("selection: 0 → 0");
});

test("canvas editor moves ime host with caret geometry", async ({ page }) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await page.keyboard.type("hello");

  const before = await playground.inputProxy.evaluate((node) => ({
    left: node.style.left,
    top: node.style.top,
    height: node.style.height,
  }));

  await playground.clickAtColumn(1.2);

  const after = await playground.inputProxy.evaluate((node) => ({
    left: node.style.left,
    top: node.style.top,
    height: node.style.height,
  }));

  expect(before.left).not.toBe(after.left);
  expect(after.height).not.toBe("1px");
});

test("canvas editor cancels composition without committing text", async ({
  page,
}) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await playground.compositionStart();
  await playground.compositionUpdate("你");

  await expect(playground.status).toContainText('composition: "你"');
  await expect(playground.status).toContainText('display: "你"');

  await playground.compositionCancel();
  await expect(playground.status).toContainText("composition: idle");
  await expect(playground.status).toContainText('text: ""');
  await expect(playground.status).toContainText('display: ""');
});

test("canvas editor supports multiline vertical movement", async ({ page }) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await page.keyboard.type("abc");
  await page.keyboard.press("Enter");
  await page.keyboard.type("def");

  await expect(playground.status).toContainText("selection: 7 → 7");

  await page.keyboard.press("ArrowUp");
  await expect(playground.status).toContainText("selection: 3 → 3");

  await page.keyboard.press("ArrowDown");
  await expect(playground.status).toContainText("selection: 7 → 7");
});

test("canvas editor supports pointer placement and drag selection", async ({
  page,
}) => {
  const playground = createPlaygroundHarness(page);

  await playground.focusEditor();
  await page.keyboard.type("hello");
  await expect(playground.status).toContainText('text: "hello"');

  await playground.clickAtColumn(1.2);
  await expect(playground.status).toContainText("selection: 1 → 1");

  await playground.dragAcrossColumns(0.2, 4.2);
  await expect(playground.status).toContainText("selection: 0 → 4");

  await page.keyboard.type("X");
  await expect(playground.status).toContainText("text length: 2");
  await expect(playground.status).toContainText('text: "Xo"');
  await expect(playground.status).toContainText("selection: 1 → 1");
});
