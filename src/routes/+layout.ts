// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://beta.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

import type { LayoutLoad } from "./$types";
import { commands } from "../generated/tauri-commands";
import { unwrap } from "$lib/utils";

export const load: LayoutLoad = async () => {
  return {
    appState: await commands.getAppState().then(unwrap),
  };
};
