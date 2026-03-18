import { expect, type Locator, type Page } from "@playwright/test";

const DEFAULT_TIMEOUT_MS = 15_000;
const PADDING_X = 24;
const LINE_Y = 30;

export type PlaygroundHarness = {
  page: Page;
  canvas: Locator;
  inputProxy: Locator;
  status: Locator;
  revision: Locator;
  docJson: Locator;
  operationLog: Locator;
  open(): Promise<void>;
  focusEditor(): Promise<void>;
  expectReady(): Promise<void>;
  snapshot<T = unknown>(): Promise<T | undefined>;
  debugSnapshot<T = unknown>(): Promise<T | undefined>;
  measuredCharWidth(): Promise<number>;
  compositionStart(data?: string): Promise<void>;
  compositionUpdate(data: string): Promise<void>;
  compositionEnd(data: string): Promise<void>;
  compositionCancel(): Promise<void>;
  insertText(text: string): Promise<void>;
  clickAtColumn(column: number): Promise<void>;
  dragAcrossColumns(from: number, to: number): Promise<void>;
};

export const createPlaygroundHarness = (page: Page): PlaygroundHarness => {
  const canvas = page.locator("#editor");
  const inputProxy = page.locator("#ime-proxy");
  const status = page.locator("#status");
  const revision = page.locator("#revision");
  const docJson = page.locator("#doc-json");
  const operationLog = page.locator("#operation-log");

  return {
    page,
    canvas,
    inputProxy,
    status,
    revision,
    docJson,
    operationLog,
    async open() {
      await page.goto("/");
    },
    async focusEditor() {
      await canvas.click();
    },
    async expectReady() {
      await expect(status).toContainText("text length: 0", {
        timeout: DEFAULT_TIMEOUT_MS,
      });
    },
    async snapshot<T = unknown>() {
      return page.evaluate(() => window.__nexPlayground?.snapshot as T | undefined);
    },
    async debugSnapshot<T = unknown>() {
      return page.evaluate(
        () => window.__nexPlayground?.debugSnapshot as T | undefined,
      );
    },
    async measuredCharWidth() {
      return page.evaluate(
        () => window.__nexPlayground?.debugSnapshot?.layout.char_width ?? 9.6,
      );
    },
    async compositionStart(data = "") {
      await dispatchComposition(page, "compositionstart", data);
    },
    async compositionUpdate(data: string) {
      await dispatchComposition(page, "compositionupdate", data);
    },
    async compositionEnd(data: string) {
      await dispatchComposition(page, "compositionend", data);
    },
    async compositionCancel() {
      await page.evaluate(() => {
        const input = document.querySelector<HTMLTextAreaElement>("#ime-proxy");
        if (!input) {
          throw new Error("IME proxy missing");
        }
        input.blur();
      });
      await this.focusEditor();
    },
    async insertText(text: string) {
      await page.keyboard.insertText(text);
    },
    async clickAtColumn(column: number) {
      const box = await canvas.boundingBox();
      if (!box) {
        throw new Error("Canvas bounding box unavailable");
      }

      const charWidth = await this.measuredCharWidth();
      await page.mouse.click(
        box.x + PADDING_X + charWidth * column,
        box.y + LINE_Y,
      );
    },
    async dragAcrossColumns(from: number, to: number) {
      const box = await canvas.boundingBox();
      if (!box) {
        throw new Error("Canvas bounding box unavailable");
      }

      const charWidth = await this.measuredCharWidth();
      await page.mouse.move(
        box.x + PADDING_X + charWidth * from,
        box.y + LINE_Y,
      );
      await page.mouse.down();
      await page.mouse.move(
        box.x + PADDING_X + charWidth * to,
        box.y + LINE_Y,
        { steps: 5 },
      );
      await page.mouse.up();
    },
  };
};

const dispatchComposition = async (
  page: Page,
  type: "compositionstart" | "compositionupdate" | "compositionend",
  data: string,
) => {
  await page.evaluate(
    ({ eventType, eventData }) => {
      const input = document.querySelector<HTMLTextAreaElement>("#ime-proxy");
      if (!input) {
        throw new Error("IME proxy missing");
      }

      input.dispatchEvent(
        new CompositionEvent(eventType, {
          data: eventData,
        }),
      );
    },
    { eventType: type, eventData: data },
  );
};
