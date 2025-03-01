import { describe, it, expect } from "vitest";
import { insertToFractionalIndexedArray } from "./utils";

// written by Claude 3.7 Sonnet
describe("insertToFractionalIndexedArray", () => {
  it("should insert an item into an empty array", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [];
    const item = { id: "1", fractionalIndex: "a0" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result).toEqual([{ id: "1", fractionalIndex: "a0" }]);
    expect(result).toBe(arr);
  });

  it("should insert items in sorted order by fractionalIndex", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [
      { id: "1", fractionalIndex: "a0" },
      { id: "3", fractionalIndex: "a2" },
    ];
    const item = { id: "2", fractionalIndex: "a1" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result).toEqual([
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ]);
  });

  it("should insert at the beginning if it has the smallest fractionalIndex", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ];
    const item = { id: "1", fractionalIndex: "a0" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result).toEqual([
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ]);
  });

  it("should insert at the end if it has the largest fractionalIndex", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
    ];
    const item = { id: "3", fractionalIndex: "a2" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result).toEqual([
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ]);
  });

  it("should not insert when an item with the same id already exists", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ];
    const item = { id: "2", fractionalIndex: "a1" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result).toEqual([
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
      { id: "3", fractionalIndex: "a2" },
    ]);
    expect(result).toBe(arr);
  });

  it("should insert when fractionalIndex is the same but id is different", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [
      { id: "1", fractionalIndex: "a0" },
      { id: "2", fractionalIndex: "a1" },
    ];
    const item = { id: "3", fractionalIndex: "a1" };

    const result = insertToFractionalIndexedArray(arr, item);

    expect(result.map((i) => i.id)).toContain("3");
    expect(result.length).toBe(3);
  });

  it("should correctly insert into a larger array", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [];
    for (let i = 0; i < 10; i++) {
      arr.push({ id: `id${i}`, fractionalIndex: `a${i}` });
    }

    const newItem = { id: "newId", fractionalIndex: "a51" };

    const result = insertToFractionalIndexedArray(arr, newItem);

    expect(result.findIndex((item) => item.id === "newId")).toBe(6);
    expect(result.length).toBe(11);
  });

  it("should not insert into a larger array if id exists", () => {
    const arr: Array<{ id: string; fractionalIndex: string }> = [];

    for (let i = 0; i < 10; i++) {
      arr.push({ id: `id${i}`, fractionalIndex: `a${i}` });
    }

    const duplicateItem = { id: "id5", fractionalIndex: "a5" };

    const result = insertToFractionalIndexedArray(arr, duplicateItem);

    expect(result.length).toBe(10);
    expect(result).toBe(arr);
  });
});
