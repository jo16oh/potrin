import { getContext, setContext } from "svelte";
import type {
  AppState,
  ClientState,
  FocusState,
  PotState,
  SidebarState,
  TabState,
  UserState,
  ViewState,
  ViewType,
  WorkspaceState,
} from "../../generated/tauri-commands";
import { commands, events } from "../../generated/tauri-commands";
import * as JsonPatch from "fast-json-patch";
const { observe, applyPatch } = JsonPatch;

const KEY = Symbol();

export type App = {
  client: Readonly<Client>;
  user: Readonly<User> | null;
  pot: Readonly<Pot> | null;
  changePot: (id: string) => Promise<void>;
};

export const App = {
  new(value: AppState): App {
    return {
      client: Client.new(value.client),
      user: value.user ? User.new(value.user) : null,
      pot: value.pot ? Pot.new(value.pot) : null,
      async changePot(id) {
        this.pot = Pot.new(await commands.fetchPotState(id));
      },
    } as const;
  },

  init(value: AppState) {
    if (getContext(KEY)) return;

    let app: App = $state(this.new(value));

    observe(app, (patch) => {
      console.log("observe");
      commands
        .updateAppState(JSON.stringify(patch))
        .then(() => {
          console.log("updated");
        })
        .catch(async (e) => {
          console.error(e);
          // app = this.new(await commands.getAppState());
        });
    });

    $inspect(app);

    void events.appStateChange.listen((e) => {
      try {
        applyPatch(app, JSON.parse(e.payload.patch));
      } catch {
        void commands.getAppState().then((r) => (app = this.new(r)));
      }
    });

    setContext(KEY, app);
  },

  state(): Readonly<App> {
    return getContext(KEY);
  },
};

type Client = {
  id: string;
};

const Client = {
  new(value: ClientState): Client {
    return {
      id: value.id,
    };
  },
};

export type User = {
  id: string;
  name: string;
  changeName: (name: string) => void;
};

const User = {
  new(value: UserState): User {
    return {
      id: value.id,
      name: value.name,
      changeName(name: string) {
        this.name = name;
      },
    };
  },
};

export const Pot = {
  new(value: PotState | { id: string; name: string }): Pot {
    return {
      id: value.id,
      name: value.name,
      workspace:
        "workspace" in value ? Workspace.new(value.workspace) : Workspace.new(),
    };
  },
};

export type Pot = {
  id: string;
  name: string;
  workspace: Workspace;
};

export type Workspace = {
  tabs: Readonly<Tab[]>;
  focus: Readonly<FocusState>;
  sidebar: Readonly<Sidebar>;
  focusTo: (to: "timeline" | "search" | number) => void;
};

const Workspace = {
  new(value?: WorkspaceState): Workspace {
    if (value) {
      return {
        tabs: value.tabs.map((v) => Tab.new(v)),
        focus: value.focus,
        sidebar: Sidebar.new(value.sidebar),
        focusTo(to: "timeline" | "search" | number) {
          this.focus =
            to === "timeline"
              ? { timeline: {} }
              : to === "search"
                ? { search: {} }
                : { tabs: { index: to } };
        },
      };
    } else {
      return {
        tabs: [],
        focus: { timeline: {} },
        sidebar: Sidebar.new(),
        focusTo(to: "timeline" | "search" | number) {
          this.focus =
            to === "timeline"
              ? { timeline: {} }
              : to === "search"
                ? { search: {} }
                : { tabs: { index: to } };
        },
      };
    }
  },
};

export type Tab = {
  views: Readonly<View[]>;
  focusedViewIdx: number;
};

const Tab = {
  new(value: TabState): Tab {
    return {
      views: value.views.map((v) => View.new(v)),
      focusedViewIdx: value.focusedViewIdx,
    };
  },
};

export type View = {
  id: string;
  viewType: Readonly<ViewType>;
  title: string;
  flexGrow: number;
};

const View = {
  new(value: ViewState): View {
    return {
      id: value.id,
      viewType: value.viewType,
      title: value.title,
      flexGrow: value.flexGrow,
    };
  },
};

export type Sidebar = {
  isFloat: boolean;
  width: number;
  toggleFloat: () => void;
  resize: (widht: number) => void;
};

const Sidebar = {
  new(value?: SidebarState): Sidebar {
    if (value) {
      return {
        isFloat: value.isFloat,
        width: value.width,
        toggleFloat() {
          this.isFloat = !this.isFloat;
        },
        resize(width: number) {
          this.width = width;
        },
      };
    } else {
      return {
        isFloat: false,
        width: 300,
        toggleFloat() {
          this.isFloat = !this.isFloat;
        },
        resize(width: number) {
          this.width = width;
        },
      };
    }
  },
};
