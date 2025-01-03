import { applyPatch, compare } from "fast-json-patch";
import {
  events,
  commands as tauriCommands,
  type FocusState,
  type PotState,
  type SidebarState,
  type TabState,
  type ViewState,
  type WorkspaceState,
} from "../../generated/tauri-commands";
import { getContext, setContext } from "svelte";
import { deepCloneOwnProperties } from "$lib/utils";

const key = Symbol();

type Commands = Pick<
  typeof tauriCommands,
  "getWorkspaceState" | "updateWorkspaceState"
>;
let commands: Commands = tauriCommands;

declare const WorkspaceBrand: unique symbol;

type WorkspaceAccessor = {
  pot: Pot;
  tabs: Tab[];
  focus: FocusState;
  sidebar: Sidebar;
} & {
  focusTo: (to: "timeline" | "search" | number) => void;
};

export type Workspace = WorkspaceAccessor & {
  [WorkspaceBrand]: unknown;
};

export const Workspace = {
  inject(cmds: Commands) {
    commands = cmds;
  },
  init(value: WorkspaceState) {
    const [workspace, setWorkspace] = Workspace.new(value);

    let prev: WorkspaceState | undefined;

    $effect(() => {
      const current = deepCloneOwnProperties(workspace);
      if (prev) {
        const diff = compare(prev, current);
        if (diff.length !== 0)
          commands.updateWorkspaceState(JSON.stringify(diff));
      }
      prev = current;
    });

    events.workspaceStateChange.listen((e) => {
      applyPatch(prev, JSON.parse(e.payload.patch));
      setWorkspace(prev!);
    });

    setContext(key, workspace);
  },
  state() {
    return getContext<Workspace>(key);
  },
  new(value: WorkspaceState): [Workspace, (value: WorkspaceState) => void] {
    let workspace = $state({
      pot: Pot.new(value.pot)[0],
      tabs: value.tabs.map((t) => Tab.new(t)[0]),
      focus: value.focus,
      sidebar: Sidebar.new(value.sidebar)[0],
    });

    const accessor: WorkspaceAccessor = {
      get pot() {
        return workspace.pot;
      },
      get tabs() {
        return workspace.tabs;
      },
      get focus() {
        return workspace.focus;
      },
      get sidebar() {
        return workspace.sidebar;
      },
      focusTo(to) {
        if (to === "timeline") {
          workspace.focus = {
            timeline: {},
          };
        } else if (to === "search") {
          workspace.focus = {
            search: {},
          };
        } else {
          workspace.focus = {
            tabs: { index: to },
          };
        }
      },
    };

    const setWorkspace = (value: WorkspaceState) => {
      workspace = {
        pot: Pot.new(value.pot)[0],
        tabs: value.tabs.map((t) => Tab.new(t)[0]),
        focus: value.focus,
        sidebar: Sidebar.new(value.sidebar)[0],
      };
    };

    return [accessor as Workspace, setWorkspace];
  },
};

declare const PotBrand: unique symbol;

type PotAccessor = PotState;
export type Pot = PotAccessor & { [PotBrand]: unknown };

const Pot = {
  new(value: PotState): [Pot, (value: PotState) => void] {
    let pot = $state({
      id: value.id,
      name: value.name,
    });

    const accessor: PotAccessor = {
      get id() {
        return pot.id;
      },
      get name() {
        return pot.name;
      },
    };

    const setPot = (value: PotState) => {
      pot = {
        id: value.id,
        name: value.name,
      };
    };

    return [accessor as Pot, setPot];
  },
};

declare const TabBrand: unique symbol;

type TabAccessor = {
  views: View[];
  focusedViewIdx: number;
};
export type Tab = TabAccessor & { [TabBrand]: unknown };

const Tab = {
  new(value: TabState): [Tab, (value: TabState) => void] {
    let tab = $state({
      views: value.views.map((v) => View.new(v)[0]),
      focusedViewIdx: value.focusedViewIdx,
    });

    const accessor: TabAccessor = {
      get views() {
        return tab.views;
      },
      get focusedViewIdx() {
        return tab.focusedViewIdx;
      },
    };

    const setTab = (value: TabState) => {
      tab = {
        views: value.views.map((v) => View.new(v)[0]),
        focusedViewIdx: value.focusedViewIdx,
      };
    };

    return [accessor as Tab, setTab];
  },
};

declare const ViewBrand: unique symbol;

export type ViewAccessor = ViewState;
type View = ViewAccessor & { [ViewBrand]: unknown };

const View = {
  new(value: ViewState): [View, (value: ViewState) => void] {
    let view = $state({
      id: value.id,
      viewType: value.viewType,
      title: value.title,
      flexGrow: value.flexGrow,
    });

    const accessor: ViewAccessor = {
      get id() {
        return view.id;
      },
      get viewType() {
        return view.viewType;
      },
      get title() {
        return view.title;
      },
      get flexGrow() {
        return view.flexGrow;
      },
    };

    const setView = (value: ViewState) => {
      view = {
        id: value.id,
        viewType: value.viewType,
        title: value.title,
        flexGrow: value.flexGrow,
      };
    };

    return [accessor as View, setView];
  },
};

declare const SidebarBrand: unique symbol;

type SidebarAccessor = SidebarState & {
  resize: (width: number) => void;
  toggleFloat: () => void;
};

export type Sidebar = SidebarAccessor & {
  [SidebarBrand]: unknown;
};

const Sidebar = {
  new(value: SidebarState): [Sidebar, (value: SidebarState) => void] {
    let sidebar = $state({
      isFloat: value.isFloat,
      width: value.width,
    });

    const accessor: SidebarAccessor = {
      get isFloat() {
        return sidebar.isFloat;
      },
      get width() {
        return sidebar.width;
      },
      resize(width: number) {
        sidebar.width = width;
      },
      toggleFloat() {
        sidebar.isFloat = !sidebar.isFloat;
      },
    };

    const setSidebar = (value: SidebarState) => {
      sidebar = {
        isFloat: value.isFloat,
        width: value.width,
      };
    };

    return [accessor as Sidebar, setSidebar];
  },
};
