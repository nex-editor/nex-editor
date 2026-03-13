import "./style.css";

type EditorSnapshot = {
  text: string;
  selection_anchor: number;
  selection_head: number;
  revision: number;
};

type WasmEditor = {
  snapshot_json(): string;
  set_text(text: string): string;
  set_selection(anchor: number, head: number): string;
  insert_text(text: string): string;
  backspace(): string;
  delete_forward(): string;
  select_all(): string;
};

type WasmModule = {
  WasmEditor: new () => WasmEditor;
};

type LineLayout = {
  text: string;
  start: number;
  end: number;
  y: number;
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

const paddingX = 24;
const paddingY = 28;
const lineHeight = 28;
const charWidth = 9.6;
const selectionColor = "#bae6fd";
const caretColor = "#0f172a";
const textColor = "#0f172a";

let editor: WasmEditor | null = null;
let snapshot: EditorSnapshot = {
  text: "",
  selection_anchor: 0,
  selection_head: 0,
  revision: 0,
};
let mouseAnchor: number | null = null;

const parseSnapshot = (raw: string): EditorSnapshot => JSON.parse(raw) as EditorSnapshot;

const layoutLines = (text: string): LineLayout[] => {
  const lines = text.split("\n");
  let offset = 0;
  return lines.map((line, index) => {
    const start = offset;
    const end = start + line.length;
    offset = end + 1;
    return {
      text: line,
      start,
      end,
      y: paddingY + index * lineHeight,
    };
  });
};

const positionToOffset = (x: number, y: number): number => {
  const lines = layoutLines(snapshot.text);
  const row = Math.max(0, Math.min(lines.length - 1, Math.floor((y - paddingY + lineHeight / 2) / lineHeight)));
  const line = lines[row] ?? { text: "", start: 0, end: 0, y: paddingY };
  const localX = Math.max(0, x - paddingX);
  const charIndex = Math.max(0, Math.min(line.text.length, Math.round(localX / charWidth)));
  return line.start + charIndex;
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

  const lines = layoutLines(snapshot.text);
  const selectionStart = Math.min(snapshot.selection_anchor, snapshot.selection_head);
  const selectionEnd = Math.max(snapshot.selection_anchor, snapshot.selection_head);

  for (const line of lines) {
    const lineSelectionStart = Math.max(selectionStart, line.start);
    const lineSelectionEnd = Math.min(selectionEnd, line.end);
    if (lineSelectionEnd > lineSelectionStart) {
      const startX = paddingX + (lineSelectionStart - line.start) * charWidth;
      const width = (lineSelectionEnd - lineSelectionStart) * charWidth;
      ctx.fillStyle = selectionColor;
      ctx.fillRect(startX, line.y - 2, width, lineHeight - 4);
    }

    ctx.fillStyle = textColor;
    ctx.fillText(line.text, paddingX, line.y);
  }

  if (selectionStart === selectionEnd) {
    const caretOffset = selectionEnd;
    let caretLine = lines[0] ?? { text: "", start: 0, end: 0, y: paddingY };
    for (const line of lines) {
      if (caretOffset >= line.start && caretOffset <= line.end) {
        caretLine = line;
        break;
      }
      if (caretOffset > line.end) {
        caretLine = line;
      }
    }
    const x = paddingX + (caretOffset - caretLine.start) * charWidth;
    ctx.fillStyle = caretColor;
    ctx.fillRect(x, caretLine.y - 1, 2, lineHeight - 6);
  }

  revision.textContent = `revision: ${snapshot.revision}`;
  status.textContent = `text length: ${snapshot.text.length} | selection: ${snapshot.selection_anchor} → ${snapshot.selection_head}`;
};

const applySnapshot = (raw: string) => {
  snapshot = parseSnapshot(raw);
  draw();
};

const ensureEditor = (): WasmEditor => {
  if (!editor) {
    throw new Error("WASM editor not initialized");
  }
  return editor;
};

const init = async () => {
  const wasm = (await import("/wasm/nex_editor_wasm.js")) as unknown as WasmModule;
  editor = new wasm.WasmEditor();
  applySnapshot(editor.snapshot_json());
  status.textContent = "ready";
};

canvas.addEventListener("mousedown", (event) => {
  canvas.focus();
  const rect = canvas.getBoundingClientRect();
  const offset = positionToOffset(event.clientX - rect.left, event.clientY - rect.top);
  mouseAnchor = offset;
  applySnapshot(ensureEditor().set_selection(offset, offset));
});

canvas.addEventListener("mousemove", (event) => {
  if (mouseAnchor === null || (event.buttons & 1) === 0) {
    return;
  }
  const rect = canvas.getBoundingClientRect();
  const offset = positionToOffset(event.clientX - rect.left, event.clientY - rect.top);
  applySnapshot(ensureEditor().set_selection(mouseAnchor, offset));
});

window.addEventListener("mouseup", () => {
  mouseAnchor = null;
});

canvas.addEventListener("keydown", (event) => {
  const instance = ensureEditor();

  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    applySnapshot(instance.select_all());
    return;
  }

  if (event.key === "Backspace") {
    event.preventDefault();
    applySnapshot(instance.backspace());
    return;
  }

  if (event.key === "Delete") {
    event.preventDefault();
    applySnapshot(instance.delete_forward());
    return;
  }

  if (event.key === "ArrowLeft") {
    event.preventDefault();
    const next = Math.max(0, Math.min(snapshot.selection_anchor, snapshot.selection_head) - 1);
    applySnapshot(instance.set_selection(next, next));
    return;
  }

  if (event.key === "ArrowRight") {
    event.preventDefault();
    const next = Math.min(snapshot.text.length, Math.max(snapshot.selection_anchor, snapshot.selection_head) + 1);
    applySnapshot(instance.set_selection(next, next));
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    applySnapshot(instance.insert_text("\n"));
    return;
  }

  if (event.key.length === 1 && !event.altKey && !event.metaKey && !event.ctrlKey) {
    event.preventDefault();
    applySnapshot(instance.insert_text(event.key));
  }
});

draw();
void init().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  status.textContent = `failed to load wasm: ${message}`;
});
