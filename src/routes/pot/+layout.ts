import type { LayoutLoad } from "./$types";
import { commands } from "generated/tauri-commands";
import { unwrap } from "$lib/utils";

export const load: LayoutLoad = async () => {
  return {
    workspaceState: await commands.getWorkspaceState().then(unwrap),
  };
};
