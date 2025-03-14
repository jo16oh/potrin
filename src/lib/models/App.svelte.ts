import { getContext, setContext } from "svelte";
import {
  commands as tauriCommands,
  events,
  type AppState,
} from "generated/tauri-commands";
import { applyPatch, compare } from "fast-json-patch";
import { getCurrent } from "@tauri-apps/api/webviewWindow";

const KEY = Symbol();

type Commands = Pick<
  typeof tauriCommands,
  "getAppState" | "updateAppState" | "openPot"
>;
let commands: Commands = tauriCommands;

export const App = {
  inject(cmds: Commands) {
    commands = cmds;
  },
  init(value: AppState) {
    let state = $state(value);
    let prev: AppState | undefined;

    let fromEvent = false;

    $effect(() => {
      // to listen deeply on the state;
      $state.snapshot(state);

      // Prevent sending diff to backend when a patch from other windows applied.
      if (fromEvent) {
        fromEvent = false;
        prev = $state.snapshot(state);
        return;
      }

      // Prevent sending diff when app loaded.
      if (!prev) {
        prev = $state.snapshot(state);
      } else {
        const diff = compare(prev, $state.snapshot(state));

        if (diff.length > 0) {
          commands
            .updateAppState(JSON.stringify(diff))
            .then(() => {
              prev = $state.snapshot(state);
            })
            .catch(() => {
              state = prev!;
            });
        }
      }
    });

    // Pass reference to this window to filter out events from this window.
    events.appStateChange(getCurrent()).listen((e) => {
      fromEvent = true;
      applyPatch(state, JSON.parse(e.payload.patch));
    });

    setContext(KEY, state);
  },
  state() {
    return getContext<AppState>(KEY);
  },
};
