import type { RenderSnapshot } from "./types";

export const formatStatus = (snapshot: RenderSnapshot): string =>
  `text length: ${snapshot.text.length} | ` +
  `selection: ${snapshot.selection_anchor} → ${snapshot.selection_head} | ` +
  `content: ${Math.round(snapshot.content_width)}×${Math.round(snapshot.content_height)} | ` +
  `text: ${JSON.stringify(snapshot.text)}`;
