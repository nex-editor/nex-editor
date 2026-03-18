export type RenderSnapshot = {
  text: string;
  display_text: string;
  selection_anchor: number;
  selection_head: number;
  revision: number;
  composition: CompositionSnapshot | null;
  viewport: {
    width: number;
    height: number;
    device_pixel_ratio: number;
  };
  content_width: number;
  content_height: number;
  scene: SceneSnapshot;
  lines: Array<{
    line_index: number;
    start: number;
    end: number;
    x: number;
    y: number;
    baseline_y: number;
    width: number;
    height: number;
    runs: Array<{
      text: string;
      start: number;
      end: number;
      x: number;
      y: number;
      baseline_y: number;
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

export type SceneSnapshot = {
  viewport: {
    width: number;
    height: number;
    device_pixel_ratio: number;
  };
  content_width: number;
  content_height: number;
  styles: PaintStyle[];
  background: PaintRect[];
  selection_rects: PaintRect[];
  composition_underlines: PaintRect[];
  text_runs: PaintTextRun[];
  caret: PaintRect | null;
};

export type CompositionSnapshot = {
  from: number;
  to: number;
  text: string;
};

export type PaintStyle = {
  id: string;
  role: PaintStyleRole;
  measurement_style_key: string | null;
};

export type PaintRect = {
  kind: "Background" | "Selection" | "Caret";
  style_id: string;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type PaintTextRun = {
  text: string;
  style_id: string;
  x: number;
  baseline_y: number;
  width: number;
  height: number;
};

export type PaintStyleRole =
  | "EditorSurface"
  | "PrimaryText"
  | "SelectionFill"
  | "CaretFill"
  | "CompositionUnderline";

export type DebugSnapshot = {
  revision: number;
  text: string;
  selection_anchor: number;
  selection_head: number;
  composition: CompositionSnapshot | null;
  layout: {
    font_family: string;
    font_size_px: number;
    char_width: number;
    line_height: number;
    caret_width: number;
    ascent: number;
    descent: number;
    text_style_key: string;
  };
  doc: unknown;
};

export type TextMeasurementEntry = {
  style_key: string;
  text: string;
  advance: number;
};

export type OperationLogEntry = {
  id: number;
  message: string;
};

export type WasmEditor = {
  snapshot_json(): string;
  debug_snapshot_json(): string;
  dispatch_json(eventJson: string): string;
};

export type WasmModule = {
  default: (moduleOrPath?: string | URL | Request) => Promise<unknown>;
  WasmEditor: new () => WasmEditor;
};

export const EMPTY_SNAPSHOT: RenderSnapshot = {
  text: "",
  display_text: "",
  selection_anchor: 0,
  selection_head: 0,
  revision: 0,
  composition: null,
  viewport: {
    width: 900,
    height: 480,
    device_pixel_ratio: 1,
  },
  content_width: 48,
  content_height: 56,
  scene: {
    viewport: {
      width: 900,
      height: 480,
      device_pixel_ratio: 1,
    },
    content_width: 48,
    content_height: 56,
    styles: [],
    background: [],
    selection_rects: [],
    composition_underlines: [],
    text_runs: [],
    caret: null,
  },
  lines: [],
  selection_rects: [],
  caret: null,
};
