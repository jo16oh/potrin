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

  describe("thread", () => {
    test("fractional index ordering", () => {
      if (!tree.child_threads) throw new Error("test failed");
      expect(tree.child_threads[0]?.title).toBe("1");
      expect(tree.child_threads[1]?.title).toBe("2");
      expect(tree.child_threads[2]?.title).toBe("3");
      expect(tree.child_threads[3]?.title).toBe("4");
      expect(tree.child_threads[4]?.title).toBe("5");
    });

    test("exclude deleted thread", () => {
      if (!tree.child_threads) throw new Error("test failed");
      expect(tree.child_threads[5]).toBeUndefined();
    });

    test("tree structuring", () => {
      expect(
        // @ts-expect-error thread nesting test
        tree.child_threads[0].child_threads[0].child_threads[0].child_threads[0]
          .child_threads[0].child_threads[0].id,
      ).toBeTruthy();
    });
  });

  describe("card", () => {
    test("card creation", async () => {
      const result = await ELECTRIC_TEST.db.cards.findMany({
        where: { thread: id },
      });
      expect(result.length).toBe(6);
    });

    test("fractional index ordering", () => {
      if (!tree.cards) throw new Error("test failed");
      expect(tree.cards[0]?.content).toBe("1");
      expect(tree.cards[1]?.content).toBe("2");
      expect(tree.cards[2]?.content).toBe("3");
      expect(tree.cards[3]?.content).toBe("4");
      expect(tree.cards[4]?.content).toBe("5");
    });

    test("exclude deleted thread", () => {
      if (!tree.child_threads) throw new Error("test failed");
      expect(tree.cards[5]).toBeUndefined();
    });

    test("tree structuring", () => {
      expect(
        // @ts-expect-error thread nesting test
        tree.child_threads[0].child_threads[0].child_threads[0].child_threads[0]
          .child_threads[0].child_threads[0]?.cards[0].id,
      ).toBeTruthy();
    });
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

  await createCard(currentID);

  return currentID;
}

async function createCard(thread: string) {
  const first = generateKeyBetween(null, null);
  const third = generateKeyBetween(first, null);
  const second = generateKeyBetween(first, third);
  const fourth = third;
  const fifth = third;
  const now = new Date();
  await ELECTRIC_TEST.db.cards.createMany({
    data: [
      {
        id: thread,
        content: "1",
        thread: thread,
        fractional_index: first,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "2",
        thread: thread,
        fractional_index: second,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "3",
        thread: thread,
        fractional_index: third,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "4",
        thread: thread,
        fractional_index: fourth,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "5",
        thread: thread,
        fractional_index: fifth,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "deleted",
        thread: thread,
        fractional_index: fifth,
        deleted: true,
        created_at: now,
        updated_at: now,
      },
    ],
  });
}
