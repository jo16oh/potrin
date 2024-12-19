import { test, expect } from "vitest";
import { Outline, type RawOutline } from "./Outline.svelte";
import { uuidv7 } from "$lib/utils";
import * as Y from "yjs";

function createTree(
  depth: number,
  parent?: RawOutline,
  currentDepth: number = 0,
): RawOutline[] {
  const outline: RawOutline = {
    id: uuidv7(),
    parentId: parent ? parent.id : null,
    doc: "",
    fractionalIndex: "a0",
    text: "",
    links: { a: [{ id: "id", text: "text" }] },
  };

  if (depth <= currentDepth) {
    return [outline];
  } else {
    return [outline, ...createTree(depth, outline, currentDepth + 1)];
  }
}

test("yjs", () => {
  const ydoc = new Y.Doc();
  const updates = [];

  ydoc.on("updateV2", (u) => {
    updates.push(u);
  });
  const yMap = ydoc.getMap("map");
  const un = new Y.UndoManager(yMap, { captureTimeout: 0 });
  const doc = new Y.Doc();
  const links = new Y.Map();
  const text = doc.getText();
  yMap.set("links", links);
  yMap.set("doc", doc);

  links.set("id", null);
  un.stopCapturing();
  text.insert(0, "as");

  const undos = [];
  const redos = [];

  un.on("stack-item-added", (i) => {
    if (i.type === "undo") {
      undos.push(i);
    } else {
      redos.push(i);
    }
  });

  un.on("stack-item-popped", (i) => {
    if (i.type === "undo") {
      undos.pop();
    } else {
      redos.pop();
    }
  });

  un.undo();

  console.log(un.undoStack.length);
  console.log(un.redoStack.length);
  expect(updates.length).toBe(4);
});

test("outline", () => {
  $effect.root(() => {
    Outline.inject(
      {
        upsertOutline: (_, __) => Promise.resolve(null),
        fetchBreadcrumbs: (_) => Promise.resolve([]),
        insertPendingYUpdate: (_, __) => Promise.resolve(null),
        fetchYUpdatesByDocId: (_) => Promise.resolve([]),
        fetchConflictingOutlineIds: (_, __, ___) => Promise.resolve([]),
      },
      "test",
    );

    const outlines = createTree(2);

    const start = performance.now();
    for (const _ of new Array(33)) {
      const outline = Outline.tree(outlines, [])[0];
    }
    console.log(performance.now() - start);

    expect(outlines.length).toBe(3);
  });
});
