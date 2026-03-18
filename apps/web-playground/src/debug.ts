import type { DebugSnapshot, OperationLogEntry } from "./types";

const EMPTY_DEBUG_TEXT = "{\n  \"type\": \"loading\"\n}";

export class DebugPanels {
  constructor(
    private readonly docJson: HTMLPreElement,
    private readonly operationLog: HTMLDivElement,
  ) {
    this.docJson.textContent = EMPTY_DEBUG_TEXT;
  }

  renderDoc(snapshot: DebugSnapshot | null): void {
    if (!snapshot) {
      this.docJson.textContent = EMPTY_DEBUG_TEXT;
      return;
    }

    this.docJson.textContent = JSON.stringify(snapshot.doc, null, 2);
  }

  renderLog(entries: OperationLogEntry[]): void {
    if (entries.length === 0) {
      this.operationLog.innerHTML = '<div class="log-empty">No operations yet.</div>';
      return;
    }

    this.operationLog.innerHTML = entries
      .map(
        (entry) =>
          `<div class="log-entry"><span class="log-id">#${entry.id}</span><span class="log-message">${escapeHtml(entry.message)}</span></div>`,
      )
      .join("");
  }
}

const escapeHtml = (text: string): string =>
  text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
