import { getContext, setContext } from "svelte";
import {
  events,
  commands as tauriCommands,
  type WorkspaceState,
} from "../../generated/tauri-commands";
import { applyPatch, compare } from "fast-json-patch";
import { watch } from "runed";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import type { DeepReadonly } from "ts-essentials";

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
    let prev: WorkspaceState | undefined;

    let fromEvent = false;

    watch(
      () => state,
      () => {
        if (fromEvent) {
          fromEvent = false;
          prev = $state.snapshot(state);
          return;
        }

        if (!prev) {
          prev = $state.snapshot(state);
        } else {
          const diff = compare(prev, state);
          if (diff.length > 0) {
            commands
              .updateWorkspaceState(JSON.stringify(diff))
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

    const updateState = (update: (state: WorkspaceState) => void) => {
      const newState = { ...state };
      update(newState);
      state = newState;
    };

    events.workspaceStateChange(getCurrent()).listen((e) => {
      fromEvent = true;
      updateState((state) => {
        applyPatch(state, JSON.parse(e.payload.patch));
      });
    });

    setContext(key, [state, updateState]);
  },
  state() {
    return getContext<
      [
        DeepReadonly<WorkspaceState>,
        (updaste: (state: WorkspaceState) => void) => void,
      ]
    >(key);
  },
};
