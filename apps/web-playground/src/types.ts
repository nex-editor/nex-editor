export type RenderSnapshot = {
  text: string;
  selection_anchor: number;
  selection_head: number;
  revision: number;
  viewport: {
    width: number;
    height: number;
    device_pixel_ratio: number;
  };
  content_width: number;
  content_height: number;
  lines: Array<{
    line_index: number;
    start: number;
    end: number;
    x: number;
    y: number;
    width: number;
    height: number;
    runs: Array<{
      text: string;
      start: number;
      end: number;
      x: number;
      y: number;
      width: number;
      height: number;
    }>;
  }>;
  selection_rects: Array<{
    x: number;
    y: number;
    width: number;
    height: number;
  }>;
  caret: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
};

export type WasmEditor = {
  snapshot_json(): string;
  dispatch_json(eventJson: string): string;
};

export type WasmModule = {
  default: (moduleOrPath?: string | URL | Request) => Promise<unknown>;
  WasmEditor: new () => WasmEditor;
};

export const EMPTY_SNAPSHOT: RenderSnapshot = {
  text: "",
  selection_anchor: 0,
  selection_head: 0,
  revision: 0,
  viewport: {
    width: 900,
    height: 480,
    device_pixel_ratio: 1,
  },
  content_width: 48,
  content_height: 56,
  lines: [],
  selection_rects: [],
  caret: null,
};
