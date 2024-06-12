import { expect, test, describe } from "vitest";
import { ELECTRIC_TEST } from "$lib/DataAccess/electric.test";
import { uuidv7 } from "uuidv7";
import { generateKeyBetween } from "fractional-indexing";
import { ThreadTree } from "./threadTree";

describe("get threadtree", async () => {
  const id = await createTree(0);
  const injectedGetLiveThreadTree = ThreadTree.live.inject({
    ELECTRIC: ELECTRIC_TEST,
  });
  const result = await injectedGetLiveThreadTree(id);
  const tree = result._unsafeUnwrap({ withStackTrace: true });
  test("fractional index ordering", () => {
    if (!tree.child_threads) throw new Error("test failed");
    expect(tree.child_threads[0]?.title).toBe("1");
    expect(tree.child_threads[1]?.title).toBe("2");
    expect(tree.child_threads[2]?.title).toBe("3");
    expect(tree.child_threads[3]?.title).toBe("4");
    expect(tree.child_threads[4]?.title).toBe("5");
  });

  test("tree structuring", () => {
    expect(
      tree.child_threads[0].child_threads[0].child_threads[0].child_threads[0]
        .child_threads[0].child_threads[0].id,
    ).toBeTruthy();
  });
});

async function createTree(depth: number, parent_thread?: string) {
  const currentID = uuidv7();
  if (depth > 6) return currentID;

  const first = generateKeyBetween(null, null);
  const third = generateKeyBetween(first, null);
  const second = generateKeyBetween(first, third);
  const fourth = third;
  const fifth = third;
  const now = new Date();

  if (parent_thread) {
    await ELECTRIC_TEST.db.threads.createMany({
      data: [
        {
          id: currentID,
          title: "1",
          parent_thread: parent_thread,
          fractional_index: first,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "2",
          parent_thread: parent_thread,
          fractional_index: second,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "3",
          parent_thread: parent_thread,
          fractional_index: third,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "4",
          parent_thread: parent_thread,
          fractional_index: fourth,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "5",
          parent_thread: parent_thread,
          fractional_index: fifth,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "deleted",
          parent_thread: parent_thread,
          fractional_index: fifth,
          deleted: true,
          created_at: now,
          updated_at: now,
        },
      ],
    });

    await createTree(depth + 1, currentID);
  } else {
    await ELECTRIC_TEST.db.threads.create({
      data: {
        id: currentID,
        title: "root",
        fractional_index: first,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
    });
    await createTree(depth + 1, currentID);
  }

  return currentID;
}
