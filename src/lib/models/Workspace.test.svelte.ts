// This code was written by Claude 3.5 Sonnet.
import { describe, test, expect, beforeEach } from "vitest";
import {
  FocusHistory,
  CloseHistory,
  type HistoryItem,
} from "./Workspace.svelte";

describe("FocusHistory", () => {
  let focusHistory: FocusHistory;

  beforeEach(() => {
    focusHistory = new FocusHistory(3);
  });

  test("should add new elements", () => {
    focusHistory.add("element1");
    expect(focusHistory.getLatest()).toBe("element1");
    expect(focusHistory.getOldest()).toBe("element1");
    expect(focusHistory.getSize()).toBe(1);
  });

  test("should remove existing elements", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const result = focusHistory.remove("element2");

    expect(result).toBe(true);
    expect(focusHistory.getSize()).toBe(2);
    expect(focusHistory.has("element2")).toBe(false);
    expect(focusHistory.getHistory()).toEqual(["element3", "element1"]);
  });

  test("should remove oldest element when exceeding maximum history size", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");
    focusHistory.add("element4");

    expect(focusHistory.getSize()).toBe(3);
    expect(focusHistory.getLatest()).toBe("element4");
    expect(focusHistory.getOldest()).toBe("element2");
    expect(focusHistory.has("element1")).toBe(false);
  });

  test("should update position when adding same ID", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element1");

    expect(focusHistory.getSize()).toBe(2);
    expect(focusHistory.getLatest()).toBe("element1");
    expect(focusHistory.getOldest()).toBe("element2");
  });

  test("getLatestN should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const latest2 = focusHistory.getLatestN(2);
    expect(latest2).toEqual(["element3", "element2"]);

    // When requested count is larger than history size
    const latest4 = focusHistory.getLatestN(4);
    expect(latest4).toEqual(["element3", "element2", "element1"]);
  });

  test("getOldestN should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    const oldest2 = focusHistory.getOldestN(2);
    expect(oldest2).toEqual(["element1", "element2"]);

    // When requested count is larger than history size
    const oldest4 = focusHistory.getOldestN(4);
    expect(oldest4).toEqual(["element1", "element2", "element3"]);
  });

  test("clearHistory should work correctly", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.clearHistory();

    expect(focusHistory.getSize()).toBe(0);
    expect(focusHistory.getLatest()).toBeNull();
    expect(focusHistory.getOldest()).toBeNull();
    expect(focusHistory.getHistory()).toEqual([]);
  });

  test("operations on empty history should work correctly", () => {
    expect(focusHistory.getLatest()).toBeNull();
    expect(focusHistory.getOldest()).toBeNull();
    expect(focusHistory.getLatestN(1)).toEqual([]);
    expect(focusHistory.getOldestN(1)).toEqual([]);
    expect(focusHistory.getSize()).toBe(0);
    expect(focusHistory.has("element1")).toBe(false);
  });

  test("getHistory should return history in correct order", () => {
    focusHistory.add("element1");
    focusHistory.add("element2");
    focusHistory.add("element3");

    expect(focusHistory.getHistory()).toEqual([
      "element3",
      "element2",
      "element1",
    ]);
  });

  test("has method should work correctly", () => {
    focusHistory.add("element1");

    expect(focusHistory.has("element1")).toBe(true);
    expect(focusHistory.has("element2")).toBe(false);
  });

  test("edge case: adding same element consecutively", () => {
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
    test("should be able to add a tab", () => {
      const tab1 = createTabState();
      history.add(tab1);
      expect(history.pop()).toBe(tab1);
    });

    test("should be able to add multiple tabs", () => {
      const tab1 = createTabState();
      const tab2 = createTabState();
      const tab3 = createTabState();

      history.add(tab1, tab2, tab3);

      expect(history.pop()).toBe(tab3);
      expect(history.pop()).toBe(tab2);
      expect(history.pop()).toBe(tab1);
    });

    test("should remove older tabs when exceeding the limit", () => {
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
    test("should return undefined when popping from empty history", () => {
      expect(history.pop()).toBeUndefined();
    });

    test("should be able to get the last added tab", () => {
      const tab1 = createTabState();
      history.add(tab1);
      expect(history.pop()).toBe(tab1);
    });

    test("should remove tab from history after popping", () => {
      const tab1 = createTabState();
      history.add(tab1);
      history.pop();
      expect(history.pop()).toBeUndefined();
    });
  });
});
