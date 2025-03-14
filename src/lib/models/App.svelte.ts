import { getContext, setContext } from "svelte";
import {
  commands as tauriCommands,
  events,
  type AppState,
} from "generated/tauri-commands";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { unwrap } from "$lib/utils";
import { watch } from "runed";
import { debounce } from "es-toolkit";

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

    let fromEvent = false;

    watch(
      () => $state.snapshot(state),
      () => {
        if (fromEvent) {
          fromEvent = false;
          return;
        }

        debounce(
          () => commands.updateAppState($state.snapshot(state)).then(unwrap),
          100,
        )();
      },
      { lazy: true },
    );

    // Pass reference to this window to filter out events from this window.
    events.appStateChange(getCurrent()).listen((e) => {
      fromEvent = true;
      state = e.payload.state;
    });

    setContext(KEY, state);
  },
  state() {
    return getContext<AppState>(KEY);
  },
};
