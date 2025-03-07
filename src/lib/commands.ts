import { unwrap } from "$lib/utils";
import {
  commands as tauriCommands,
  type FetchTimelineOption,
  type UUIDv7Base64URL,
} from "generated/tauri-commands";
import { Outline } from "./models/Outline.svelte";
import { TimelineDay } from "./models/Timeline.svelte";

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

export async function fetchTimeline(
  option: FetchTimelineOption,
): Promise<TimelineDay | null> {
  return await tauriCommands
    .fetchTimeline(option)
    .then(unwrap)
    .then((day) => (day ? new TimelineDay(day) : null));
}
