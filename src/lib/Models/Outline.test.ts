import { uuidv7 } from "uuidv7";
import { test, expect } from "vitest";
import { Outline, type RawOutline } from "./Outline";
import * as Y from "yjs";
import { UndoManager } from "yjs";

test("ydoc", async () => {
  const doc = new Y.Doc();
  const ytext = doc.getText("text!");
  const ytext2 = doc.getText("parent");
  const undoManager = new UndoManager(ytext);
  const updates: Uint8Array[] = [];

  doc.on("updateV2", (update) => {
    updates.push(update);
  });

  ytext.observe((event) => {
    // console.log(event.);
    console.log(event);
  });

  ytext.insert(0, "y");
  ytext.insert(1, "y");
  ytext2.insert(0, "y");
  ytext2.insert(1, "y");

  //
  // console.log(ytext.toString());
  //
  // const mergedUpdate = Y.mergeUpdatesV2(updates);
  // const doc2 = new Y.Doc();
  // const ytext2 = doc2.getText();
  // for (const u of updates) {
  //   Y.applyUpdateV2(doc2, u);
  // }
  // Y.applyUpdateV2(doc2, mergedUpdate);
  // console.log("2", ytext2.toString());

  expect(true).toBeTruthy();
});

test("construction", () => {
  const data = dummyData();
  console.log(data.length);

  for (let i = 0; i < 10; i++) {
    const d = dummyData();
    console.time("constructOutline");
    Outline.treeFromArray(d);
    console.timeEnd("constructOutline");
  }

  const [outline] = Outline.treeFromArray(data);

  expect(
    // @ts-expect-error tree structuring test
    outline.children[0].children[0].children[0].children[0].children[0]
      .children[0].parent.parent.parent.parent.parent.parent,
  ).toBe(outline);
});

function dummyData(): RawOutline[] {
  const root: RawOutline = {
    id: uuidv7(),
    parent: null,
    text: "root",
  };

  const result = createChildren(root.id);

  return [root, ...result];

  function createChildren(
    parent: string,
    maxDepth: number = 9,
    maxChildren: number = 3,
    currentDepth: number = 0,
  ): RawOutline[] {
    if (maxDepth <= currentDepth) return [];

    const children: RawOutline[] = [];
    for (let i = 0; i < maxChildren; i++) {
      children.push({
        id: uuidv7(),
        parent: parent,
        text: "text here text here text here text here",
      });
    }

    const grandChildren: RawOutline[] = [];
    children.forEach((c) => {
      createChildren(c.id, maxDepth, maxChildren, currentDepth + 1).forEach(
        (gc) => {
          grandChildren.push(gc);
        },
      );
    });

    return [...children, ...grandChildren];
  }
}
