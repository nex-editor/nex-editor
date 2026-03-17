import type { RenderSnapshot } from "./types";

const selectionColor = "#bae6fd";
const caretColor = "#0f172a";
const textColor = "#0f172a";

export class CanvasRenderer {
  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly ctx: CanvasRenderingContext2D,
  ) {}

  draw(snapshot: RenderSnapshot): void {
    const dpr = window.devicePixelRatio || 1;
    const cssWidth = this.canvas.clientWidth || 900;
    const cssHeight = 480;
    this.canvas.width = cssWidth * dpr;
    this.canvas.height = cssHeight * dpr;
    this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.ctx.clearRect(0, 0, cssWidth, cssHeight);

    this.ctx.fillStyle = "#ffffff";
    this.ctx.fillRect(0, 0, cssWidth, cssHeight);

    this.ctx.font = '18px "IBM Plex Mono", monospace';
    this.ctx.textBaseline = "top";

    for (const rect of snapshot.selection_rects) {
      this.ctx.fillStyle = selectionColor;
      this.ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
    }

    for (const line of snapshot.lines) {
      for (const run of line.runs) {
        this.ctx.fillStyle = textColor;
        this.ctx.fillText(run.text, run.x, run.y);
      }
    }

    if (snapshot.caret) {
      this.ctx.fillStyle = caretColor;
      this.ctx.fillRect(
        snapshot.caret.x,
        snapshot.caret.y,
        snapshot.caret.width,
        snapshot.caret.height,
      );
    }
  }
}
