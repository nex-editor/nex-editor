import "./style.css";

import { WasmEditorBridge } from "./bridge";
import {
  keyEvent,
  pointerDownEvent,
  pointerMoveEvent,
  pointerUpEvent,
  viewportEvent,
} from "./input";
import { CanvasRenderer } from "./renderer";
import { mountShell } from "./shell";
import { formatStatus } from "./status";
import { EMPTY_SNAPSHOT, type RenderSnapshot } from "./types";

const { canvas, status, revision } = mountShell();

const ctx = canvas.getContext("2d");
if (!ctx) {
  throw new Error("Canvas 2D context unavailable");
}

const bridge = new WasmEditorBridge();
const renderer = new CanvasRenderer(canvas, ctx);

let snapshot: RenderSnapshot = EMPTY_SNAPSHOT;
let isPointerSelecting = false;

const updateStatus = () => {
  revision.textContent = `revision: ${snapshot.revision}`;
  status.textContent = formatStatus(snapshot);
};

const renderSnapshot = (next: RenderSnapshot) => {
  snapshot = next;
  renderer.draw(snapshot);
  updateStatus();
};

const dispatch = (event: unknown) => {
  if (!bridge.isReady()) {
    status.textContent = "loading wasm...";
    return snapshot;
  }
  return bridge.dispatch(event);
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

canvas.addEventListener("mousedown", (event) => {
  canvas.focus();
  const rect = canvas.getBoundingClientRect();
  isPointerSelecting = true;
  renderSnapshot(dispatch(pointerDownEvent(event, rect)));
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

canvas.addEventListener("keydown", (event) => {
  const editorEvent = keyEvent(event);
  if (!editorEvent) {
    return;
  }

  renderSnapshot(dispatch(editorEvent));
  event.preventDefault();
});

window.addEventListener("resize", () => {
  syncViewport();
});

renderSnapshot(snapshot);
void bridge
  .init()
  .then((initialSnapshot) => {
    renderSnapshot(initialSnapshot);
    syncViewport();
  })
  .catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    status.textContent = `failed to load wasm: ${message}`;
  });
