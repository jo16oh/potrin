import type {
  AppState,
  Base64String,
  ClientState,
  PotState,
  TabState,
  UserState,
  WorkspaceState,
} from "../../generated/tauri-commands";
import { commands } from "../../generated/tauri-commands";

export class App {
  private static _state: App | undefined = $state();

  private _client: Client;
  private _user: User | null;
  private _pot: Pot | null;
  private _workspace: Workspace | null;

  private constructor(state: AppState) {
    this._client = new Client(state.client);
    this._user = state.user ? new User(state.user) : null;
    this._pot = state.pot ? new Pot(state.pot) : null;
    this._workspace = state.workspace ? new Workspace(state.workspace) : null;
  }

  static async init(state: AppState) {
    if (!this._state) this._state = new App(state);
  }

  static get state() {
    if (!this._state) throw new Error("App is not initialized");
    return this._state;
  }
}

export class Client {
  private _id: Base64String;

  constructor(value: ClientState) {
    this._id = value.id;
  }

  get id() {
    return this._id;
  }
}

export class User {
  private _id: Base64String;
  private _name: string;

  constructor(user: UserState) {
    this._id = user.id;
    this._name = user.name;
  }

  get id() {
    return this._id;
  }

  get name() {
    return this._name;
  }

  set id(value: Base64String) {
    commands.updateUserState({ Id: value });
    this._id = value;
  }

  set name(value: Base64String) {
    commands.updateUserState({ Name: value });
    this._name = value;
  }
}

export class Pot {
  private _id: Base64String;
  private _sync: boolean;

  constructor(pot: PotState) {
    this._id = pot.id;
    this._sync = pot.sync;
  }

  get id() {
    return this._id;
  }

  get sync() {
    return this._sync;
  }

  set id(value: Base64String) {
    commands.updatePotState({ Id: value });
    this._id = value;
  }

  set sync(value: boolean) {
    commands.updatePotState({ Sync: value });
    this.sync = value;
  }
}

export class Workspace {
  private _tabs: Tab[];
  private _focused_tab_idx: number | null;

  constructor(workspace: WorkspaceState) {
    const tabs: Tab[] = [];

    for (const [index, tab] of workspace.tabs.entries()) {
      tabs.push(new Tab(tab, tabs, index));
    }

    this._tabs = tabs;
    this._focused_tab_idx = workspace.focused_tab_idx;
  }

  get tabs() {
    return this._tabs;
  }

  get focused_tab_idx() {
    return this._focused_tab_idx;
  }

  set tabs(value: Tab[]) {
    commands.updateWorkspaceState({ Tabs: value });
    this._tabs = value;
  }

  set focused_tab_idx(value: number | null) {
    commands.updateWorkspaceState({ FocusedTabIdx: value });
    this._focused_tab_idx = value;
  }
}

export class Tab {
  private parentArray: Tab[];
  private index: number;
  private _id: Base64String;
  private _view: string;
  private _scroll_pos: number;

  constructor(tab: TabState, array: Tab[], index: number) {
    this.parentArray = array;
    this.index = index;
    this._id = tab.id;
    this._view = tab.view;
    this._scroll_pos = tab.scroll_pos;
  }

  get id() {
    return this._id;
  }

  get view() {
    return this._view;
  }

  get scroll_pos() {
    return this._scroll_pos;
  }

  close() {
    this.parentArray.splice(this.index, 1);
  }
}
