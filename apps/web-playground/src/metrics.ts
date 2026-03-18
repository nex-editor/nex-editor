export const EDITOR_FONT_SIZE_PX = 18;
export const EDITOR_FONT_FAMILY = '"IBM Plex Mono", monospace';
export const EDITOR_FONT = `${EDITOR_FONT_SIZE_PX}px ${EDITOR_FONT_FAMILY}`;

export type TextMetricsPayload = {
  font_family: string;
  font_size_px: number;
  char_width: number;
  line_height: number;
  caret_width: number;
  ascent: number;
  descent: number;
};

export type LayoutDebugMetrics = {
  font_family: string;
  font_size_px: number;
  line_height: number;
  text_style_key: string;
};

export type TextMeasurementEntry = {
  style_key: string;
  text: string;
  advance: number;
};

const segmenter =
  typeof Intl !== "undefined" && "Segmenter" in Intl
    ? new Intl.Segmenter(undefined, { granularity: "grapheme" })
    : null;

export class TextMeasurementCache {
  private readonly cache = new Map<string, Map<string, number>>();

  async measureMissing(
    text: string,
    metrics: LayoutDebugMetrics,
  ): Promise<TextMeasurementEntry[]> {
    if ("fonts" in document) {
      await document.fonts.ready;
    }

    const styleKey = metrics.text_style_key;
    const entries = this.cache.get(styleKey) ?? new Map<string, number>();
    this.cache.set(styleKey, entries);

    const missing = Array.from(
      new Set(
        segmentTextUnits(text).filter(
          (unit) => unit !== "\n" && !entries.has(unit),
        ),
      ),
    );
    if (missing.length === 0) {
      return [];
    }

    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas 2D context unavailable for text measurement");
    }

    ctx.font = `${metrics.font_size_px}px ${metrics.font_family}`;
    ctx.textBaseline = "alphabetic";

    return missing.map((unit) => {
      const advance = Number(ctx.measureText(unit).width.toFixed(4));
      entries.set(unit, advance);
      return {
        style_key: styleKey,
        text: unit,
        advance,
      };
    });
  }
}

export const measureEditorTextMetrics = async (): Promise<TextMetricsPayload> => {
  if ("fonts" in document) {
    await document.fonts.ready;
  }

  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Canvas 2D context unavailable for text measurement");
  }

  ctx.font = EDITOR_FONT;
  ctx.textBaseline = "alphabetic";

  const sample = "MMMMMMMMMM";
  const measure = ctx.measureText(sample);
  const charWidth = measure.width / sample.length;
  const ascent =
    measure.actualBoundingBoxAscent > 0
      ? measure.actualBoundingBoxAscent
      : EDITOR_FONT_SIZE_PX * 0.8;
  const descent =
    measure.actualBoundingBoxDescent > 0
      ? measure.actualBoundingBoxDescent
      : EDITOR_FONT_SIZE_PX * 0.25;
  const lineHeight = Math.ceil(ascent + descent + 8);

  return {
    font_family: EDITOR_FONT_FAMILY,
    font_size_px: EDITOR_FONT_SIZE_PX,
    char_width: Number(charWidth.toFixed(4)),
    line_height: lineHeight,
    caret_width: 2,
    ascent: Number(ascent.toFixed(4)),
    descent: Number(descent.toFixed(4)),
  };
};

export const segmentTextUnits = (text: string): string[] => {
  if (segmenter) {
    return Array.from(segmenter.segment(text), (entry) => entry.segment);
  }

  return Array.from(text);
};
