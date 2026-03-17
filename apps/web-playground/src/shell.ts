export type PlaygroundShell = {
  canvas: HTMLCanvasElement;
  status: HTMLDivElement;
  revision: HTMLSpanElement;
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

  return { canvas, status, revision };
};
