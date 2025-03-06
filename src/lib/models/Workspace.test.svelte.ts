// This code was written by Claude 3.5 Sonnet.
import { describe, it, expect, beforeEach } from "vitest";
import {
  FocusHistory,
  CloseHistory,
  type HistoryItem,
  ViewHistory,
} from "./Workspace.svelte";
import type { ViewState } from "../../generated/tauri-commands";

describe("FocusHistory", () => {
  let focusHistory: FocusHistory;

  beforeEach(() => {
    focusHistory = new FocusHistory(3);
  });

  it("should add new elements", () => {
    focusHistory.add("element1");
    expect(focusHistory.getLatest()).toBe("element1");
    expect(focusHistory.getOldest()).toBe("element1");
    expect(focusHistory.getSize()).toBe(1);
  });

  it("should remove existing elements", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const result = focusHistory.remove("element2");

    expect(result).toBe(true);
    expect(focusHistory.getSize()).toBe(2);
    expect(focusHistory.has("element2")).toBe(false);
    expect(focusHistory.getHistory()).toEqual(["element3", "element1"]);
  });

  it("should remove oldest element when exceeding maximum history size", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");
    focusHistory.add("element4");

    expect(focusHistory.getSize()).toBe(3);
    expect(focusHistory.getLatest()).toBe("element4");
    expect(focusHistory.getOldest()).toBe("element2");
    expect(focusHistory.has("element1")).toBe(false);
  });

  it("should update position when adding same ID", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element1");

    expect(focusHistory.getSize()).toBe(2);
    expect(focusHistory.getLatest()).toBe("element1");
    expect(focusHistory.getOldest()).toBe("element2");
  });

  it("getLatestN should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const latest2 = focusHistory.getLatestN(2);
    expect(latest2).toEqual(["element3", "element2"]);

    // When requested count is larger than history size
    const latest4 = focusHistory.getLatestN(4);
    expect(latest4).toEqual(["element3", "element2", "element1"]);
  });

  it("getOldestN should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const oldest2 = focusHistory.getOldestN(2);
    expect(oldest2).toEqual(["element1", "element2"]);

    // When requested count is larger than history size
    const oldest4 = focusHistory.getOldestN(4);
    expect(oldest4).toEqual(["element1", "element2", "element3"]);
  });

  it("clearHistory should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.clearHistory();

    expect(focusHistory.getSize()).toBe(0);
    expect(focusHistory.getLatest()).toBeNull();
    expect(focusHistory.getOldest()).toBeNull();
    expect(focusHistory.getHistory()).toEqual([]);
  });

  it("operations on empty history should work correctly", () => {
    expect(focusHistory.getLatest()).toBeNull();
    expect(focusHistory.getOldest()).toBeNull();
    expect(focusHistory.getLatestN(1)).toEqual([]);
    expect(focusHistory.getOldestN(1)).toEqual([]);
    expect(focusHistory.getSize()).toBe(0);
    expect(focusHistory.has("element1")).toBe(false);
  });

  it("getHistory should return history in correct order", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    expect(focusHistory.getHistory()).toEqual([
      "element3",
      "element2",
      "element1",
    ]);
  });

  it("has method should work correctly", () => {
    focusHistory.add("element1");

    expect(focusHistory.has("element1")).toBe(true);
    expect(focusHistory.has("element2")).toBe(false);
  });

  it("edge case: adding same element consecutively", () => {
    focusHistory.add("element1");
    focusHistory.add("element1");
    focusHistory.add("element1");

    expect(focusHistory.getSize()).toBe(1);
    expect(focusHistory.getHistory()).toEqual(["element1"]);
  });
});

describe("CloseHistory", () => {
  let history: CloseHistory;

  beforeEach(() => {
    history = new CloseHistory(3);
  });

  function createTabState(): HistoryItem {
    return {
      type: "tab",
      index: 0,
      value: {
        id: crypto.randomUUID(),
        views: [],
        focusedViewId: null,
      },
    };
  }

  describe("add", () => {
    it("should be able to add a tab", () => {
      const tab1 = createTabState();
      history.add(tab1);
      expect(history.pop()).toBe(tab1);
    });

    it("should be able to add multiple tabs", () => {
      const tab1 = createTabState();
      const tab2 = createTabState();
      const tab3 = createTabState();

      history.add(tab1, tab2, tab3);

      expect(history.pop()).toBe(tab3);
      expect(history.pop()).toBe(tab2);
      expect(history.pop()).toBe(tab1);
    });

    it("should remove older tabs when exceeding the limit", () => {
      const tab1 = createTabState();
      const tab2 = createTabState();
      const tab3 = createTabState();
      const tab4 = createTabState();

      history.add(tab1, tab2, tab3, tab4);

      // First tab should be removed due to limit
      expect(history.pop()).toBe(tab4);
      expect(history.pop()).toBe(tab3);
      expect(history.pop()).toBe(tab2);
      expect(history.pop()).toBeUndefined();
    });
  });

  describe("pop", () => {
    it("should return undefined when popping from empty history", () => {
      expect(history.pop()).toBeUndefined();
    });

    it("should be able to get the last added tab", () => {
      const tab1 = createTabState();
      history.add(tab1);
      expect(history.pop()).toBe(tab1);
    });

    it("should remove tab from history after popping", () => {
      const tab1 = createTabState();
      history.add(tab1);
      history.pop();
      expect(history.pop()).toBeUndefined();
    });
  });
});

describe("ViewHistory", () => {
  let viewHistory: ViewHistory;
  let mockState1: ViewState;
  let mockState2: ViewState;

  function createMockViewState(
    id: string,
  ): Extract<ViewState, { type: "cards" }> {
    return {
      id: id,
      type: "cards",
      title: "",
      outlineId: crypto.randomUUID(),
      viewWidthRatio: 1,
      scrollPosition: 0,
      focusPosition: {
        id: null,
        position: null,
      },
    };
  }

  beforeEach(() => {
    viewHistory = new ViewHistory();
    mockState1 = createMockViewState("id");
    mockState2 = createMockViewState("id");
  });

  describe("addPrev", () => {
    it("should add first state to history", () => {
      viewHistory.addPrev(mockState1);
      expect(viewHistory.hasPrev(mockState1.id)).toBe(true);
    });

    it("should add multiple states to history", () => {
      viewHistory.addPrev(mockState1);
      viewHistory.addPrev(mockState2);
      expect(viewHistory.hasPrev(mockState1.id)).toBe(true);
    });

    it("should clear next history when adding prev", () => {
      viewHistory.addPrev(mockState1);
      viewHistory.getPrev(mockState2);
      viewHistory.addPrev(mockState2);
      expect(viewHistory.hasNext(mockState1.id)).toBe(false);
    });
  });

  describe("getPrev", () => {
    it("should return undefined for non-existent history", () => {
      const result = viewHistory.getPrev(mockState1);
      expect(result).toBeUndefined();
    });

    it("should return previous state and move current to next", () => {
      viewHistory.addPrev(mockState1);
      const prev = viewHistory.getPrev(mockState2);

      expect(prev).toEqual(mockState1);
      expect(viewHistory.hasNext(mockState1.id)).toBe(true);
    });
  });

  describe("getNext", () => {
    it("should return undefined for non-existent history", () => {
      const result = viewHistory.getNext(mockState1);
      expect(result).toBeUndefined();
    });

    it("should return next state and move current to prev", () => {
      viewHistory.addPrev(mockState1);
      viewHistory.getPrev(mockState2);

      const next = viewHistory.getNext(mockState2);
      expect(next).toEqual(mockState2);
      expect(viewHistory.hasPrev(mockState1.id)).toBe(true);
    });
  });

  describe("remove", () => {
    it("should remove history for given id", () => {
      viewHistory.addPrev(mockState1);
      viewHistory.remove(mockState1.id);

      expect(viewHistory.hasPrev(mockState1.id)).toBe(false);
      expect(viewHistory.hasNext(mockState1.id)).toBe(false);
    });
  });

  describe("hasPrev", () => {
    it("should return false for non-existent history", () => {
      expect(viewHistory.hasPrev("nonexistent")).toBe(false);
    });

    it("should return true when prev history exists", () => {
      viewHistory.addPrev(mockState1);
      expect(viewHistory.hasPrev(mockState1.id)).toBe(true);
    });
  });

  describe("hasNext", () => {
    it("should return false for non-existent history", () => {
      expect(viewHistory.hasNext("nonexistent")).toBe(false);
    });

    it("should return true when next history exists", () => {
      viewHistory.addPrev(mockState1);
      viewHistory.addPrev(mockState2);
      viewHistory.getPrev(mockState2);
      expect(viewHistory.hasNext(mockState1.id)).toBe(true);
    });
  });

  describe("limit", () => {
    it("should pop history when reach limit", () => {
      let viewHistory = new ViewHistory(3);
      const s1 = createMockViewState("id");
      const s2 = createMockViewState("id");
      const s3 = createMockViewState("id");
      const s4 = createMockViewState("id");
      const s5 = createMockViewState("id");
      viewHistory.addPrev(s1);
      viewHistory.addPrev(s2);
      viewHistory.addPrev(s3);
      viewHistory.addPrev(s4);
      // @ts-ignore
      expect(viewHistory.getPrev(s5)?.outlineId).toBe(s4.outlineId);
      // @ts-ignore
      expect(viewHistory.getPrev(s4)?.outlineId).toBe(s3.outlineId);
      // @ts-ignore
      expect(viewHistory.getPrev(s3)?.outlineId).toBe(s2.outlineId);
      // @ts-ignore
      expect(viewHistory.getPrev(s2)?.outlineId).toBe(undefined);
      expect(viewHistory.hasPrev(s2.id)).toBe(false);
    });
  });
});
