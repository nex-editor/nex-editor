export type PlaygroundShell = {
  canvas: HTMLCanvasElement;
  inputProxy: HTMLTextAreaElement;
  status: HTMLDivElement;
  revision: HTMLSpanElement;
  docJson: HTMLPreElement;
  operationLog: HTMLDivElement;
};

export const mountShell = (): PlaygroundShell => {
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
          <canvas id="editor" width="900" height="480"></canvas>
          <textarea id="ime-proxy" class="ime-proxy" spellcheck="false" autocapitalize="off" autocomplete="off" autocorrect="off"></textarea>
        </div>
        <div class="footer" id="status">loading wasm...</div>
      </section>
      <section class="debug-grid">
        <section class="panel debug-panel">
          <div class="debug-header">
            <h2>Document JSON</h2>
            <p>Serialized Rust document tree for the current editor state.</p>
          </div>
          <pre class="debug-pre" id="doc-json"></pre>
        </section>
        <section class="panel debug-panel">
          <div class="debug-header">
            <h2>Operation Log</h2>
            <p>Browser events forwarded to the runtime and their resulting state.</p>
          </div>
          <div class="log-list" id="operation-log"></div>
        </section>
      </section>
    </div>
  `;

  const canvas = document.querySelector<HTMLCanvasElement>("#editor");
  const inputProxy = document.querySelector<HTMLTextAreaElement>("#ime-proxy");
  const status = document.querySelector<HTMLDivElement>("#status");
  const revision = document.querySelector<HTMLSpanElement>("#revision");
  const docJson = document.querySelector<HTMLPreElement>("#doc-json");
  const operationLog = document.querySelector<HTMLDivElement>("#operation-log");

  if (!canvas || !inputProxy || !status || !revision || !docJson || !operationLog) {
    throw new Error("Missing UI elements");
  }

  return { canvas, inputProxy, status, revision, docJson, operationLog };
};
