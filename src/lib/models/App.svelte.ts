import { getContext, setContext } from "svelte";
import {
  commands as tauriCommands,
  events,
  type AppState,
  type UserState,
} from "../../generated/tauri-commands";
import { applyPatch, compare } from "fast-json-patch";
import { deepCloneOwnProperties } from "$lib/utils";

const key = Symbol();

type Commands = Pick<
  typeof tauriCommands,
  "getAppState" | "updateAppState" | "openPot"
>;
let commands: Commands = tauriCommands;

declare const AppBrand: unique symbol;

type AppAccessor = AppState & {
  openPot: (pot: { id: string; name: string }) => void;
};

export type App = AppAccessor & { [AppBrand]: unknown };

export const App = {
  inject(cmds: Commands) {
    commands = cmds;
  },
  init(value: AppState) {
    const [app, setApp] = App.new(value);

    let prev: AppState | undefined;

    $effect(() => {
      const current = deepCloneOwnProperties(app);
      if (prev) {
        const diff = compare(prev, current);
        if (diff.length !== 0) commands.updateAppState(JSON.stringify(diff));
      }
      prev = current;
    });

    events.appStateChange.listen((e) => {
      applyPatch(prev, JSON.parse(e.payload.patch));
      setApp(prev!);
    });

    setContext(key, app);
  },
  state() {
    return getContext<App>(key);
  },
  new(value: AppState): [App, (v: AppState) => void] {
    let [user, setUser] = value.user ? User.new(value.user) : [null, null];

    let app = $state({
      clientId: value.clientId,
      user: user,
      pots: value.pots,
      setting: value.setting,
    });

    const accessor: AppAccessor = {
      get clientId() {
        return app.clientId;
      },
      get user() {
        return app.user;
      },
      get pots() {
        return app.pots;
      },
      get setting() {
        return app.setting;
      },
      set user(u: UserState | null) {
        if (u && setUser) {
          setUser(u);
          app.user = user;
        } else if (u) {
          [user, setUser] = User.new(u);
          app.user = user;
        } else {
          app.user = null;
        }
      },
      openPot(pot: { id: string; name: string }) {
        app.pots[pot.id] = pot.name;
        commands.openPot(pot.id, pot.name);
      },
    };

    const setApp = (value: AppState) => {
      app = {
        clientId: value.clientId,
        user: value.user ? User.new(value.user)[0] : null,
        pots: value.pots,
        setting: value.setting,
      };
    };

    return [accessor as App, setApp];
  },
};

declare const UserBrand: unique symbol;

type UserAccessor = UserState;

export type User = UserAccessor & { [UserBrand]: unknown };

const User = {
  new(value: UserState): [User, (u: UserState) => void] {
    // validation
    let user = $state({
      id: value.id,
      name: value.name,
    });

    const accessor: UserAccessor = {
      get id() {
        return user.id;
      },
      get name() {
        return user.name;
      },
      set name(n) {
        user.name = n;
      },
    };

    const setUser = (value: UserState) => {
      // validation
      user = {
        id: value.id,
        name: value.name,
      };
    };

    return [accessor as User, setUser];
  },
};
