import { getContext, setContext } from "svelte";
import {
  commands as tauriCommands,
  events,
  type AppState,
} from "../../generated/tauri-commands";
import { applyPatch, compare } from "fast-json-patch";
import type { DeepReadonly } from "ts-essentials";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { watch } from "runed";

const key = Symbol();

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

    watch(
      () => state,
      // This callback takes both the current and previous state value,
      // but the previous value is the shallow copy of the state,
      // so managing previous state manually here.
      () => {
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
          const diff = compare(prev, state);

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
      },
    );

    // Change values inside the update function to avoid
    // `ownership_invalid_mutation` warning.
    const updateState = (update: (state: AppState) => void) => {
      const newState = { ...state };
      update(newState);
      state = newState;
    };

    // Pass reference to this window to filter out events from this window.
    events.appStateChange(getCurrent()).listen((e) => {
      fromEvent = true;
      updateState((state) => {
        applyPatch(state, JSON.parse(e.payload.patch));
      });
    });

    setContext(key, [state, updateState]);
  },
  state() {
    return getContext<
      [DeepReadonly<AppState>, (update: (state: AppState) => void) => void]
    >(key);
  },
};
