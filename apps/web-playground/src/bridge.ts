import type { DebugSnapshot, RenderSnapshot, WasmEditor, WasmModule } from "./types";
import { EMPTY_SNAPSHOT } from "./types";

const parseSnapshot = (raw: string): RenderSnapshot =>
  JSON.parse(raw) as RenderSnapshot;
const parseDebugSnapshot = (raw: string): DebugSnapshot =>
  JSON.parse(raw) as DebugSnapshot;

export class WasmEditorBridge {
  private editor: WasmEditor | null = null;
  private ready = false;
  private snapshot: RenderSnapshot = EMPTY_SNAPSHOT;
  private debugSnapshot: DebugSnapshot | null = null;

  async init(): Promise<RenderSnapshot> {
    const wasm =
      (await import("./wasm/nex_editor_wasm.js")) as unknown as WasmModule;
    await wasm.default();
    this.editor = new wasm.WasmEditor();
    this.ready = true;
    this.snapshot = parseSnapshot(this.editor.snapshot_json());
    this.debugSnapshot = parseDebugSnapshot(this.editor.debug_snapshot_json());
    return this.snapshot;
  }

  isReady(): boolean {
    return this.ready && this.editor !== null;
  }

  getSnapshot(): RenderSnapshot {
    return this.snapshot;
  }

  getDebugSnapshot(): DebugSnapshot | null {
    return this.debugSnapshot;
  }

  dispatch(event: unknown): RenderSnapshot {
    if (!this.editor || !this.ready) {
      return this.snapshot;
    }

    this.snapshot = parseSnapshot(this.editor.dispatch_json(JSON.stringify(event)));
    this.debugSnapshot = parseDebugSnapshot(this.editor.debug_snapshot_json());
    return this.snapshot;
  }
}
