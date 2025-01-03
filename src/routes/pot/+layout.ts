import type { LayoutLoad } from "./$types";
import { commands } from "../../generated/tauri-commands";

export const load: LayoutLoad = async () => {
  return {
    workspaceState: await commands.getWorkspaceState(),
  };
};
