import "./style.css";

type RenderSnapshot = {
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

type WasmEditor = {
  snapshot_json(): string;
  dispatch_json(eventJson: string): string;
};

type WasmModule = {
  default: (moduleOrPath?: string | URL | Request) => Promise<unknown>;
  WasmEditor: new () => WasmEditor;
};

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("Missing app root");
}

app.innerHTML = `
  <div class="shell">
    <section class="panel">
      <div class="header">
        <h1 class="title">nex-editor playground</h1>
        <p class="subtitle">WASM-driven plain text editor snapshot rendered on canvas.</p>
      </div>
      <div class="toolbar">
        <span>Keys: typing, Enter, Backspace, Delete, Cmd/Ctrl+A</span>
        <span id="revision">revision: 0</span>
      </div>
      <div class="canvas-wrap">
        <canvas id="editor" width="900" height="480" tabindex="0"></canvas>
      </div>
      <div class="footer" id="status">loading wasm...</div>
    </section>
  </div>
`;

const canvas = document.querySelector<HTMLCanvasElement>("#editor");
const status = document.querySelector<HTMLDivElement>("#status");
const revision = document.querySelector<HTMLSpanElement>("#revision");

if (!canvas || !status || !revision) {
  throw new Error("Missing UI elements");
}

const ctx = canvas.getContext("2d");
if (!ctx) {
  throw new Error("Canvas 2D context unavailable");
}

const selectionColor = "#bae6fd";
const caretColor = "#0f172a";
const textColor = "#0f172a";

let editor: WasmEditor | null = null;
let snapshot: RenderSnapshot = {
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
let isPointerSelecting = false;
let isReady = false;

const parseSnapshot = (raw: string): RenderSnapshot =>
  JSON.parse(raw) as RenderSnapshot;

const applyRenderSnapshot = (next: RenderSnapshot) => {
  snapshot = next;
  draw();
};

const dispatch = (event: unknown): RenderSnapshot => {
  const instance = ensureEditor();
  if (!instance) {
    return snapshot;
  }
  return parseSnapshot(instance.dispatch_json(JSON.stringify(event)));
};

const draw = () => {
  const dpr = window.devicePixelRatio || 1;
  const cssWidth = canvas.clientWidth || 900;
  const cssHeight = 480;
  canvas.width = cssWidth * dpr;
  canvas.height = cssHeight * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, cssWidth, cssHeight);

  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, cssWidth, cssHeight);

  ctx.font = '18px "IBM Plex Mono", monospace';
  ctx.textBaseline = "top";

  for (const rect of snapshot.selection_rects) {
    ctx.fillStyle = selectionColor;
    ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
  }

  for (const line of snapshot.lines) {
    for (const run of line.runs) {
      ctx.fillStyle = textColor;
      ctx.fillText(run.text, run.x, run.y);
    }
  }

  if (snapshot.caret) {
    ctx.fillStyle = caretColor;
    ctx.fillRect(
      snapshot.caret.x,
      snapshot.caret.y,
      snapshot.caret.width,
      snapshot.caret.height,
    );
  }

  revision.textContent = `revision: ${snapshot.revision}`;
  status.textContent = `text length: ${snapshot.text.length} | selection: ${snapshot.selection_anchor} → ${snapshot.selection_head} | content: ${Math.round(snapshot.content_width)}×${Math.round(snapshot.content_height)} | text: ${JSON.stringify(snapshot.text)}`;
};

const applySnapshot = (raw: string) => applyRenderSnapshot(parseSnapshot(raw));

const ensureEditor = (): WasmEditor | null => {
  if (!editor || !isReady) {
    status.textContent = "loading wasm...";
    return null;
  }
  return editor;
};

const init = async () => {
  const wasm =
    (await import("./wasm/nex_editor_wasm.js")) as unknown as WasmModule;
  await wasm.default();
  editor = new wasm.WasmEditor();
  isReady = true;
  applySnapshot(editor.snapshot_json());
  applyRenderSnapshot(
    dispatch({
      ResizeViewport: {
        width: canvas.clientWidth || 900,
        height: 480,
        device_pixel_ratio: window.devicePixelRatio || 1,
      },
    }),
  );
};

canvas.addEventListener("mousedown", (event) => {
  canvas.focus();
  const rect = canvas.getBoundingClientRect();
  isPointerSelecting = true;
  applyRenderSnapshot(
    dispatch({
      PointerDown: {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
        button: "Primary",
        modifiers: {
          shift: event.shiftKey,
          alt: event.altKey,
          meta: event.metaKey,
          ctrl: event.ctrlKey,
        },
        click_count: event.detail,
      },
    }),
  );
});

canvas.addEventListener("mousemove", (event) => {
  if (!isPointerSelecting || (event.buttons & 1) === 0) {
    return;
  }
  const rect = canvas.getBoundingClientRect();
  applyRenderSnapshot(
    dispatch({
      PointerMove: {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
        modifiers: {
          shift: event.shiftKey,
          alt: event.altKey,
          meta: event.metaKey,
          ctrl: event.ctrlKey,
        },
      },
    }),
  );
});

window.addEventListener("mouseup", (event) => {
  if (isPointerSelecting) {
    const rect = canvas.getBoundingClientRect();
    applyRenderSnapshot(
      dispatch({
        PointerUp: {
          x: event.clientX - rect.left,
          y: event.clientY - rect.top,
          button: "Primary",
          modifiers: {
            shift: event.shiftKey,
            alt: event.altKey,
            meta: event.metaKey,
            ctrl: event.ctrlKey,
          },
        },
      }),
    );
  }
  isPointerSelecting = false;
});

canvas.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("SelectAll"));
    return;
  }

  if (event.key === "Backspace") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("Backspace"));
    return;
  }

  if (event.key === "Delete") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("DeleteForward"));
    return;
  }

  if (event.key === "ArrowLeft") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("MoveCaretLeft"));
    return;
  }

  if (event.key === "ArrowRight") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("MoveCaretRight"));
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("MoveCaretUp"));
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    applyRenderSnapshot(dispatch("MoveCaretDown"));
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    applyRenderSnapshot(dispatch({ InsertText: { text: "\n" } }));
    return;
  }

  if (
    event.key.length === 1 &&
    !event.altKey &&
    !event.metaKey &&
    !event.ctrlKey
  ) {
    event.preventDefault();
    applyRenderSnapshot(dispatch({ InsertText: { text: event.key } }));
  }
});

window.addEventListener("resize", () => {
  applyRenderSnapshot(
    dispatch({
      ResizeViewport: {
        width: canvas.clientWidth || 900,
        height: 480,
        device_pixel_ratio: window.devicePixelRatio || 1,
      },
    }),
  );
});

draw();
void init().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  status.textContent = `failed to load wasm: ${message}`;
});
