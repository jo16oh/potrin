import { getContext, setContext } from "svelte";
import {
  events,
  commands as tauriCommands,
  type WorkspaceState,
} from "../../generated/tauri-commands";
import { applyPatch, observe } from "fast-json-patch";

const key = Symbol();

type Commands = Pick<
  typeof tauriCommands,
  "getWorkspaceState" | "updateWorkspaceState"
>;
let commands: Commands = tauriCommands;

export const Workspace = {
  inject(cmds: Commands) {
    commands = cmds;
  },
  init(value: WorkspaceState) {
    let state = $state(value);
    let prev = $state.snapshot(state);

    observe(state, (patches) => {
      commands
        .updateWorkspaceState(JSON.stringify(patches))
        .then(() => {
          prev = $state.snapshot(state);
        })
        .catch((e) => {
          state = prev;
          console.error(e);
        });
    });

    events.workspaceStateChange.listen((e) => {
      applyPatch(state, JSON.parse(e.payload.patch));
    });

    setContext(key, state);
  },
  state() {
    return getContext<WorkspaceState>(key);
  },
};
