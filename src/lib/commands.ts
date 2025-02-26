import { unwrap } from "$lib/utils";
import {
  commands as tauriCommands,
  type UUIDv7Base64URL,
} from "../generated/tauri-commands";
import { Outline } from "./models/Outline.svelte";

export async function fetchTree(id: UUIDv7Base64URL, depth: number | null) {
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
