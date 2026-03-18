import type { RenderSnapshot } from "./types";

export const formatStatus = (snapshot: RenderSnapshot): string =>
  `text length: ${snapshot.text.length} | ` +
  `selection: ${snapshot.selection_anchor} → ${snapshot.selection_head} | ` +
  `composition: ${snapshot.composition ? JSON.stringify(snapshot.composition.text) : "idle"} | ` +
  `content: ${Math.round(snapshot.content_width)}×${Math.round(snapshot.content_height)} | ` +
  `text: ${JSON.stringify(snapshot.text)} | ` +
  `display: ${JSON.stringify(snapshot.display_text)}`;
