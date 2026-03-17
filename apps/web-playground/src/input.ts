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

  if (event.key === "Enter") {
    return { InsertText: { text: "\n" } };
  }

  if (
    event.key.length === 1 &&
    !event.altKey &&
    !event.metaKey &&
    !event.ctrlKey
  ) {
    return { InsertText: { text: event.key } };
  }

  return null;
};
