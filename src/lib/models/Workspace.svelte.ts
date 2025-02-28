import { getContext, setContext, untrack } from "svelte";
import {
  events,
  commands as tauriCommands,
  type TabState,
  type ViewState,
  type WorkspaceState,
} from "../../generated/tauri-commands";
import { applyPatch, compare } from "fast-json-patch";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { SvelteMap } from "svelte/reactivity";
import { Outline } from "./Outline.svelte";
import { Paragraph } from "./Paragraph.svelte";

const KEY = Symbol();

type Commands = Pick<
  typeof tauriCommands,
  "getWorkspaceState" | "updateWorkspaceState"
>;
let commands: Commands = tauriCommands;

export class Workspace {
  static inject(cmds: Commands) {
    commands = cmds;
  }

  static init(value: WorkspaceState) {
    const instance = new Workspace(value);
    setContext(KEY, instance);

    let prev: WorkspaceState | undefined;

    let fromEvent = false;

    $effect(() => {
      const _ = instance.#state;

      if (fromEvent) {
        fromEvent = false;
        prev = $state.snapshot(instance.#state);
        return;
      }

      if (!prev) {
        prev = $state.snapshot(instance.#state);
      } else {
        const diff = compare(prev, $state.snapshot(instance.#state));
        if (diff.length > 0) {
          commands
            .updateWorkspaceState(JSON.stringify(diff))
            .then(() => {
              prev = $state.snapshot(instance.#state);
            })
            .catch(() => {
              instance.#state = prev!;
            });
        }
      }
    });

    events.workspaceStateChange(getCurrent()).listen((e) => {
      fromEvent = true;
      applyPatch(instance.#state, JSON.parse(e.payload.patch));
    });

    $effect(() => {
      if (instance.state.focusedTabId)
        untrack(() => instance.#focusHistory.add(instance.state.focusedTabId!));
    });
  }

  static get current() {
    return getContext<Workspace>(KEY);
  }

  get state() {
    return this.#state;
  }

  #state = $state<WorkspaceState>()!;
  #focusHistory = new FocusHistory(100);
  #closeHistory = new CloseHistory(100);

  private constructor(value: WorkspaceState) {
    this.#state = value;
  }

  isTabLoaded(tabId: string) {
    return this.#focusHistory.has(tabId);
  }

  currentTab() {
    return this.#state.focusedTabId
      ? (this.#state.tabs.find((t) => t.id === this.#state.focusedTabId) ??
          null)
      : null;
  }

  closeView(tab: TabState, tabIdx: number, view: ViewState, viewIdx: number) {
    const tabs = this.state.tabs;

    const closedView = tab.views.splice(viewIdx, 1)[0];
    if (tab.views.length === 0) {
      const closedTab = tabs.splice(tabIdx, 1)[0];
      if (closedTab) {
        this.#focusHistory.remove(closedTab.id);
        this.#closeHistory.add({
          type: "tab",
          index: tabIdx,
          value: closedTab,
        });
      }

      if (tabs.length === 0) {
        this.state.focusedTabId = null;
      } else if (this.state.focusedTabId === tab.id) {
        if (tabIdx !== tabs.length) {
          this.state.focusedTabId = tabs[tabIdx]?.id ?? null;
        } else if (tabIdx === tabs.length) {
          this.state.focusedTabId = tabs[tabIdx - 1]?.id ?? null;
        }
      }
    } else if (tab.focusedViewId === view.id) {
      if (closedView) {
        this.#closeHistory.add({
          type: "view",
          index: viewIdx,
          tabId: tab.id,
          value: closedView,
        });
      }
      tab.focusedViewId = tab.views[tab.views.length - 1]?.id ?? null;
    }
  }
}

class Node {
  id: string;
  prev: Node | null;
  next: Node | null;

  constructor(id: string) {
    this.id = id;
    this.prev = null;
    this.next = null;
  }
}

export class FocusHistory {
  private maxHistory: number;
  private size: number = 0;
  private idToNode = $state(new SvelteMap<string, Node>());
  private head: Node | null = null;
  private tail: Node | null = null;

  constructor(maxHistory: number) {
    this.maxHistory = maxHistory;
  }

  getLatest(): string | null {
    return this.head ? this.head.id : null;
  }

  getOldest(): string | null {
    return this.tail ? this.tail.id : null;
  }

  getLatestN(n: number): string[] {
    const result: string[] = [];
    let current = this.head;
    let count = 0;

    while (current && count < n) {
      result.push(current.id);
      current = current.next;
      count++;
    }

    return result;
  }

  getOldestN(n: number): string[] {
    const result: string[] = [];
    let current = this.tail;
    let count = 0;

    while (current && count < n) {
      result.push(current.id);
      current = current.prev;
      count++;
    }

    return result;
  }

  getSize(): number {
    return this.size;
  }

  has(id: string): boolean {
    return this.idToNode.has(id);
  }

  add(id: string): void {
    let node = this.idToNode.get(id);

    if (node) {
      this.#removeNode(node);
      this.size--;
    } else {
      node = new Node(id);
    }

    this.#addToFront(node);
    this.idToNode.set(id, node);
    this.size++;

    if (this.size > this.maxHistory) {
      this.#removeTail();
    }
  }

  remove(id: string): boolean {
    const node = this.idToNode.get(id);
    if (!node) {
      return false;
    }

    this.#removeNode(node);
    this.idToNode.delete(id);
    this.size--;

    return true;
  }

  getHistory(): string[] {
    const history: string[] = [];
    let current = this.head;
    while (current) {
      history.push(current.id);
      current = current.next;
    }
    return history;
  }

  clearHistory(): void {
    this.idToNode.clear();
    this.head = null;
    this.tail = null;
    this.size = 0;
  }

  #removeNode(node: Node): void {
    if (node.prev) {
      node.prev.next = node.next;
    } else {
      this.head = node.next;
    }

    if (node.next) {
      node.next.prev = node.prev;
    } else {
      this.tail = node.prev;
    }
  }

  #addToFront(node: Node): void {
    node.next = this.head;
    node.prev = null;

    if (this.head) {
      this.head.prev = node;
    }
    this.head = node;

    if (!this.tail) {
      this.tail = node;
    }
  }

  #removeTail(): void {
    if (!this.tail) return;

    this.idToNode.delete(this.tail.id);
    this.tail = this.tail.prev;
    if (this.tail) {
      this.tail.next = null;
    } else {
      this.head = null;
    }
    this.size--;
  }
}

export type HistoryItem =
  | { type: "tab"; index: number; value: TabState }
  | { type: "view"; index: number; tabId: string; value: ViewState };

export class CloseHistory {
  #history: HistoryItem[] = [];
  #limit: number;

  constructor(limit: number) {
    this.#limit = limit;
  }

  add(...history: HistoryItem[]) {
    this.#history.push(...history);
    if (this.#history.length > this.#limit) {
      this.#history.shift();
    }
  }

  pop() {
    return this.#history.pop();
  }
}

type ViewHistoryItem = {
  prev: ViewState[];
  next: ViewState[];
};

export class ViewHistory {
  #limit = 100;
  #map = new SvelteMap<string, ViewHistoryItem>();

  constructor(limit?: number) {
    if (limit) this.#limit = limit;
  }

  addPrev(prevState: ViewState) {
    const history = this.#map.get(prevState.id);
    if (history) {
      if (this.#limit <= history.prev.length) {
        history.prev.shift();
      }
      history.prev.push($state.snapshot(prevState));
      history.next.length = 0;
      this.#map.set(prevState.id, { ...history });
    } else {
      const h = { prev: [$state.snapshot(prevState)], next: [] };
      this.#map.set(prevState.id, h);
    }
  }

  getPrev(currentState: ViewState) {
    const history = this.#map.get(currentState.id);
    if (history) {
      history?.next.unshift($state.snapshot(currentState));
      const prev = history.prev.pop();
      this.#map.set(currentState.id, { ...history });
      return prev;
    } else {
      return undefined;
    }
  }

  getNext(currentState: ViewState) {
    const history = this.#map.get(currentState.id);
    if (history) {
      history?.prev.push($state.snapshot(currentState));
      const next = history.next.shift();
      this.#map.set(currentState.id, { ...history });
      return next;
    } else {
      return undefined;
    }
  }

  remove(id: string) {
    this.#map.delete(id);
  }

  hasPrev(id: string) {
    const prev = this.#map.get(id)?.prev;
    return prev ? prev.length !== 0 : false;
  }

  hasNext(id: string) {
    const next = this.#map.get(id)?.next;
    return next ? next.length !== 0 : false;
  }
}

class ViewMethods {
  #history = $state(new ViewHistory(100));

  open(current: ViewState, next: ViewState) {
    this.#history.addPrev(current);
    Object.assign(current, next);
  }

  back(current: ViewState) {
    const prev = this.#history.getPrev(current);
    if (prev) Object.assign(current, prev);

    // workaround for keep focus position
    if ("focusPosition" in current) {
      const focusPos = { ...current.focusPosition };
      setTimeout(() => {
        current.focusPosition = focusPos;
      }, 16);
    }
  }

  forward(current: ViewState) {
    const next = this.#history.getNext(current);
    if (next) Object.assign(current, next);

    // workaround for keep focus position
    if ("focusPosition" in current) {
      const focusPos = { ...current.focusPosition };
      setTimeout(() => {
        current.focusPosition = focusPos;
      }, 16);
    }
  }

  hasPrev(id: string) {
    return this.#history.hasPrev(id);
  }

  hasNext(id: string) {
    return this.#history.hasNext(id);
  }

  async save(current: ViewState) {
    if ("focusPosition" in current && current.focusPosition.id) {
      await Outline.buffer.get(current.focusPosition.id)?.save();
      await Paragraph.buffer.get(current.focusPosition.id)?.save();
    }
  }
}

export type View = ViewState;
export const View = new ViewMethods();
