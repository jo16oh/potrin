import { uuidv7 } from "uuidv7";
import { test, expect } from "vitest";
import { Outline, type RawOutline } from "./Outline";

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
