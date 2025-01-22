import { getContext, setContext } from "svelte";
import {
  commands as tauriCommands,
  events,
  type AppState,
} from "../../generated/tauri-commands";
import { applyPatch, observe } from "fast-json-patch";

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
    let prev = $state.snapshot(state);

    observe(state, (patches) => {
      commands
        .updateAppState(JSON.stringify(patches))
        .then(() => {
          prev = $state.snapshot(state);
        })
        .catch((e) => {
          state = prev;
          console.error(e);
        });
    });

    events.appStateChange.listen((e) => {
      applyPatch(state, JSON.parse(e.payload.patch));
    });

    setContext(key, state);
  },
  state() {
    return getContext<AppState>(key);
  },
};
