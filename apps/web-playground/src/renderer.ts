import type {
  PaintRect,
  PaintStyle,
  PaintStyleRole,
  PaintTextRun,
  RenderSnapshot,
} from "./types";
import { EDITOR_FONT } from "./metrics";

const ROLE_FILL: Record<PaintStyleRole, string> = {
  EditorSurface: "#ffffff",
  PrimaryText: "#0f172a",
  SelectionFill: "#bae6fd",
  CaretFill: "#0f172a",
  CompositionUnderline: "#2563eb",
};

export class CanvasRenderer {
  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly ctx: CanvasRenderingContext2D,
  ) {}

  draw(snapshot: RenderSnapshot): void {
    const dpr = window.devicePixelRatio || 1;
    const cssWidth = this.canvas.clientWidth || 900;
    const cssHeight = 480;
    const styles = new Map(
      snapshot.scene.styles.map((style) => [style.id, style] as const),
    );
    this.canvas.width = cssWidth * dpr;
    this.canvas.height = cssHeight * dpr;
    this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.ctx.clearRect(0, 0, cssWidth, cssHeight);
    this.ctx.font = EDITOR_FONT;
    this.ctx.textBaseline = "alphabetic";

    for (const rect of snapshot.scene.background) {
      this.drawRect(rect, styles);
    }

    for (const rect of snapshot.scene.selection_rects) {
      this.drawRect(rect, styles);
    }

    for (const rect of snapshot.scene.composition_underlines) {
      this.drawRect(rect, styles);
    }

    for (const run of snapshot.scene.text_runs) {
      this.drawTextRun(run, styles);
    }

    if (snapshot.scene.caret) {
      this.drawRect(snapshot.scene.caret, styles);
    }
  }

  private drawRect(rect: PaintRect, styles: Map<string, PaintStyle>): void {
    this.ctx.fillStyle = this.resolveFill(rect.style_id, styles);
    this.ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
  }

  private drawTextRun(run: PaintTextRun, styles: Map<string, PaintStyle>): void {
    this.ctx.fillStyle = this.resolveFill(run.style_id, styles);
    this.ctx.fillText(run.text, run.x, run.baseline_y);
  }

  private resolveFill(styleId: string, styles: Map<string, PaintStyle>): string {
    const style = styles.get(styleId);
    if (!style) {
      return ROLE_FILL.PrimaryText;
    }

    return ROLE_FILL[style.role];
  }
}
