const modifiersFromEvent = (
  event: MouseEvent | KeyboardEvent,
) => ({
  shift: event.shiftKey,
  alt: event.altKey,
  meta: event.metaKey,
  ctrl: event.ctrlKey,
});

export const viewportEvent = (
  width: number,
  height: number,
  devicePixelRatio: number,
) => ({
  ResizeViewport: {
    width,
    height,
    device_pixel_ratio: devicePixelRatio,
  },
});

export const textMetricsEvent = (metrics: {
  font_family: string;
  font_size_px: number;
  char_width: number;
  line_height: number;
  caret_width: number;
  ascent: number;
  descent: number;
}) => ({
  SetTextMetrics: metrics,
});

export const textMeasurementsEvent = (entries: {
  style_key: string;
  text: string;
  advance: number;
}[]) => ({
  SetTextMeasurements: { entries },
});

export const compositionStartEvent = () => "CompositionStart";

export const compositionUpdateEvent = (text: string) => ({
  CompositionUpdate: { text },
});

export const compositionEndEvent = (text: string) => ({
  CompositionEnd: { text },
});

export const compositionCancelEvent = () => "CompositionCancel";

export const pointerDownEvent = (
  event: MouseEvent,
  rect: DOMRect,
) => ({
  PointerDown: {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
    button: "Primary",
    modifiers: modifiersFromEvent(event),
    click_count: event.detail,
  },
});

export const pointerMoveEvent = (
  event: MouseEvent,
  rect: DOMRect,
) => ({
  PointerMove: {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
    modifiers: modifiersFromEvent(event),
  },
});

export const pointerUpEvent = (
  event: MouseEvent,
  rect: DOMRect,
) => ({
  PointerUp: {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
    button: "Primary",
    modifiers: modifiersFromEvent(event),
  },
});

export const keyEvent = (event: KeyboardEvent): unknown | null => {
  if (event.isComposing || event.key === "Process") {
    return null;
  }

  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "a") {
    return "SelectAll";
  }

  const mappedKeys: Record<string, unknown> = {
    Backspace: "Backspace",
    Delete: "DeleteForward",
    ArrowLeft: "MoveCaretLeft",
    ArrowRight: "MoveCaretRight",
    ArrowUp: "MoveCaretUp",
    ArrowDown: "MoveCaretDown",
  };

  const mappedEvent = mappedKeys[event.key];
  if (mappedEvent) {
    return mappedEvent;
  }

  return null;
};

export const beforeInputEvent = (event: InputEvent): unknown | null => {
  if (event.isComposing) {
    return null;
  }

  if (event.inputType === "insertText" && event.data) {
    return { InsertText: { text: event.data } };
  }

  if (event.inputType === "insertLineBreak") {
    return { InsertText: { text: "\n" } };
  }

  return null;
};
