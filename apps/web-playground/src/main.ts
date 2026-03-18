import "./style.css";

import { WasmEditorBridge } from "./bridge";
import { DebugPanels } from "./debug";
import {
  beforeInputEvent,
  compositionCancelEvent,
  compositionEndEvent,
  compositionStartEvent,
  compositionUpdateEvent,
  keyEvent,
  pointerDownEvent,
  pointerMoveEvent,
  pointerUpEvent,
  textMeasurementsEvent,
  textMetricsEvent,
  viewportEvent,
} from "./input";
import { measureEditorTextMetrics, TextMeasurementCache } from "./metrics";
import { CanvasRenderer } from "./renderer";
import { mountShell } from "./shell";
import { formatStatus } from "./status";
import {
  EMPTY_SNAPSHOT,
  type DebugSnapshot,
  type OperationLogEntry,
  type RenderSnapshot,
} from "./types";

declare global {
  interface Window {
    __nexPlayground?: {
      snapshot: RenderSnapshot;
      debugSnapshot: DebugSnapshot | null;
    };
  }
}

const { canvas, inputProxy, status, revision, docJson, operationLog } = mountShell();

const ctx = canvas.getContext("2d");
if (!ctx) {
  throw new Error("Canvas 2D context unavailable");
}

const bridge = new WasmEditorBridge();
const renderer = new CanvasRenderer(canvas, ctx);
const debugPanels = new DebugPanels(docJson, operationLog);
const textMeasurementCache = new TextMeasurementCache();

let snapshot: RenderSnapshot = EMPTY_SNAPSHOT;
let debugSnapshot: DebugSnapshot | null = null;
let isPointerSelecting = false;
let isComposing = false;
let nextLogId = 1;
let textMeasurementSyncPending = false;
let textMeasurementSyncRunning = false;
const operationEntries: OperationLogEntry[] = [];

const syncInputProxyPosition = () => {
  const caret = snapshot.caret ?? snapshot.scene.caret;
  const layout = debugSnapshot?.layout;

  if (caret) {
    inputProxy.style.left = `${caret.x}px`;
    inputProxy.style.top = `${caret.y}px`;
    inputProxy.style.height = `${Math.max(caret.height, 1)}px`;
  } else if (snapshot.scene.composition_underlines[0]) {
    const underline = snapshot.scene.composition_underlines[0];
    inputProxy.style.left = `${underline.x}px`;
    inputProxy.style.top = `${underline.y}px`;
    inputProxy.style.height = "1px";
  } else {
    inputProxy.style.left = "0px";
    inputProxy.style.top = "0px";
    inputProxy.style.height = "1px";
  }

  if (layout) {
    inputProxy.style.font = `${layout.font_size_px}px ${layout.font_family}`;
    inputProxy.style.lineHeight = `${layout.line_height}px`;
  }
};

const updateStatus = () => {
  revision.textContent = `revision: ${snapshot.revision}`;
  status.textContent = formatStatus(snapshot);
  debugPanels.renderDoc(debugSnapshot);
  debugPanels.renderLog(operationEntries);
  syncInputProxyPosition();
  window.__nexPlayground = {
    snapshot,
    debugSnapshot,
  };
};

const runTextMeasurementSync = async () => {
  if (textMeasurementSyncRunning) {
    textMeasurementSyncPending = true;
    return;
  }

  textMeasurementSyncRunning = true;
  textMeasurementSyncPending = true;

  try {
    while (textMeasurementSyncPending) {
      textMeasurementSyncPending = false;

      if (!bridge.isReady() || !debugSnapshot?.layout) {
        continue;
      }

      const entries = await textMeasurementCache.measureMissing(
        snapshot.display_text,
        debugSnapshot.layout,
      );
      if (entries.length === 0) {
        continue;
      }

      renderSnapshot(dispatch(textMeasurementsEvent(entries)), {
        scheduleTextMeasurementSync: false,
      });
    }
  } finally {
    textMeasurementSyncRunning = false;
    if (textMeasurementSyncPending) {
      void runTextMeasurementSync();
    }
  }
};

const renderSnapshot = (
  next: RenderSnapshot,
  options: { scheduleTextMeasurementSync?: boolean } = {},
) => {
  snapshot = next;
  debugSnapshot = bridge.getDebugSnapshot();
  renderer.draw(snapshot);
  updateStatus();

  if (options.scheduleTextMeasurementSync ?? true) {
    void runTextMeasurementSync();
  }
};

const pushOperationLog = (message: string) => {
  operationEntries.unshift({
    id: nextLogId,
    message,
  });
  nextLogId += 1;
  operationEntries.splice(24);
};

const snapshotSummary = (value: RenderSnapshot) =>
  `revision=${value.revision} selection=${value.selection_anchor}->${value.selection_head} text=${JSON.stringify(value.text)}`;

const describeEvent = (event: unknown): string => {
  if (typeof event === "string") {
    return event;
  }

  if (!event || typeof event !== "object") {
    return String(event);
  }

  const [name, payload] = Object.entries(event)[0] ?? ["Unknown", null];
  if (!payload || typeof payload !== "object") {
    return name;
  }

  return `${name} ${JSON.stringify(payload)}`;
};

const dispatch = (event: unknown) => {
  if (!bridge.isReady()) {
    status.textContent = "loading wasm...";
    return snapshot;
  }
  const next = bridge.dispatch(event);
  pushOperationLog(`${describeEvent(event)} -> ${snapshotSummary(next)}`);
  return next;
};

const syncViewport = () => {
  renderSnapshot(
    dispatch(
      viewportEvent(
        canvas.clientWidth || 900,
        480,
        window.devicePixelRatio || 1,
      ),
    ),
  );
};

const syncTextMetrics = async () => {
  const metrics = await measureEditorTextMetrics();
  renderSnapshot(dispatch(textMetricsEvent(metrics)));
};

const focusInputProxy = () => {
  inputProxy.focus();
};

canvas.addEventListener("mousedown", (event) => {
  event.preventDefault();
  focusInputProxy();
  const rect = canvas.getBoundingClientRect();
  isPointerSelecting = true;
  renderSnapshot(dispatch(pointerDownEvent(event, rect)));
});

canvas.addEventListener("click", () => {
  focusInputProxy();
});

canvas.addEventListener("mousemove", (event) => {
  if (!isPointerSelecting || (event.buttons & 1) === 0) {
    return;
  }

  const rect = canvas.getBoundingClientRect();
  renderSnapshot(dispatch(pointerMoveEvent(event, rect)));
});

window.addEventListener("mouseup", (event) => {
  if (!isPointerSelecting) {
    return;
  }

  const rect = canvas.getBoundingClientRect();
  renderSnapshot(dispatch(pointerUpEvent(event, rect)));
  isPointerSelecting = false;
});

inputProxy.addEventListener("keydown", (event) => {
  const editorEvent = keyEvent(event);
  if (!editorEvent) {
    return;
  }

  renderSnapshot(dispatch(editorEvent));
  event.preventDefault();
});

inputProxy.addEventListener("beforeinput", (event) => {
  const editorEvent = beforeInputEvent(event);
  if (!editorEvent) {
    return;
  }

  renderSnapshot(dispatch(editorEvent));
  event.preventDefault();
});

inputProxy.addEventListener("compositionstart", () => {
  isComposing = true;
  renderSnapshot(dispatch(compositionStartEvent()));
});

inputProxy.addEventListener("compositionupdate", (event) => {
  renderSnapshot(dispatch(compositionUpdateEvent(event.data ?? inputProxy.value)));
});

inputProxy.addEventListener("compositionend", (event) => {
  isComposing = false;
  renderSnapshot(dispatch(compositionEndEvent(event.data ?? inputProxy.value)));
  inputProxy.value = "";
});

inputProxy.addEventListener("blur", () => {
  if (!isComposing) {
    return;
  }
  isComposing = false;
  renderSnapshot(dispatch(compositionCancelEvent()));
  inputProxy.value = "";
});

window.addEventListener("resize", () => {
  syncViewport();
});

renderSnapshot(snapshot);
pushOperationLog("bootstrap shell");
updateStatus();
focusInputProxy();
void bridge
  .init()
  .then(async (initialSnapshot) => {
    renderSnapshot(initialSnapshot);
    pushOperationLog(`init runtime -> ${snapshotSummary(initialSnapshot)}`);
    await syncTextMetrics();
    syncViewport();
  })
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    status.textContent = `failed to load wasm: ${message}`;
    pushOperationLog(`init error -> ${message}`);
    updateStatus();
  });
