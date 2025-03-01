import { unwrap } from "$lib/utils";
import {
  commands as tauriCommands,
  type UUIDv7Base64URL,
} from "../generated/tauri-commands";
import { Outline } from "./models/Outline.svelte";
import { Paragraph } from "./models/Paragraph.svelte";

export async function fetchTree(
  id: UUIDv7Base64URL,
  depth: number | null,
): Promise<Outline> {
  const outline = Outline.buffer.get(id);
  if (outline) {
    tauriCommands
      .fetchTree(id, depth)
      .then(unwrap)
      .then(([o, p]) => Outline.tree(o, p, id));
    return outline;
  } else {
    return await tauriCommands
      .fetchTree(id, depth)
      .then(unwrap)
      .then(([o, p]) => Outline.tree(o, p, id));
  }
}

export async function fetchTimeline(from: number): Promise<Outline[]> {
  return await tauriCommands
    .fetchTimeline(from)
    .then(unwrap)
    .then(([rawOutlines, rawParagraphs]) => {
      const outlines = rawOutlines.map((o) => Outline.from(o));
      const outlineMap = new Map(outlines.map((o) => [o.id, o]));
      const paragraphs = rawParagraphs
        .map((p) => {
          const outline = outlineMap.get(p.outlineId);
          const paragraph = outline ? Paragraph.from(p, outline) : undefined;
          if (outline && paragraph) outline.insertParagraph(paragraph);
          return paragraph;
        })
        .filter((p) => p !== undefined);

      // Sort the outlines along with the paragraphs
      // because the paragraphs are already sorted by createdAt by the sql.
      return new Set(paragraphs.map((p) => p.outlineId))
        .keys()
        .map((id) => outlineMap.get(id))
        .filter((o) => o !== undefined)
        .toArray();
    });
}
