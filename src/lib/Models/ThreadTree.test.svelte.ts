import { expect, test, describe } from "vitest";
import { ELECTRIC_TEST } from "$lib/DataAccess/electric.test";
import { uuidv7 } from "uuidv7";
import { generateKeyBetween } from "fractional-indexing";
import { ThreadTree } from "./ThreadTree.svelte";

describe.sequential("threadtree", () => {
  describe("cache", async () => {
    const injectedGetLiveThreadTree = ThreadTree.getLiveTree.inject({
      ELECTRIC: ELECTRIC_TEST,
    });

    const id = uuidv7();
    const cache = JSON.stringify({ id: id, title: "cache", cards: [] });
    localStorage.setItem(id, cache);

    await ELECTRIC_TEST.db.threads.create({
      data: {
        id: id,
        fractional_index: "a",
        title: "title",
        deleted: false,
        created_at: new Date(),
        updated_at: new Date(),
      },
    });

    await new Promise((resolve) => setTimeout(resolve, 100));

    let result: ReturnType<typeof injectedGetLiveThreadTree>;
    $effect.root(() => {
      result = injectedGetLiveThreadTree(id);
    });

    const [unsubscribe, liveTree] = result._unsafeUnwrap({
      withStackTrace: true,
    });

    // initial state of the liveTree to be cache
    test("initial state of the liveTree to be cache", async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(liveTree.state?.title).toBe("cache");
    });

    test("unsubscribe / resubscribe / react to update", async () => {
      const subscribe = unsubscribe();
      await ELECTRIC_TEST.db.threads.update({
        where: { id: id },
        data: { title: "updated" },
      });
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(liveTree.state?.title).toBe("cache");

      subscribe();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(liveTree.state?.title).toBe("updated");
    });

    test("cache should be removed when received db updates", async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(JSON.parse(localStorage.getItem(id))).toBe(null);
    });

    test("cache should be updated along with the state change", async () => {
      liveTree.state.title = "3";
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(JSON.parse(localStorage.getItem(id))?.title).toBe("3");
    });
  });

  describe("get threadtree", async () => {
    test("get threadtree", async () => {
      const id = await createTree(0);
      const injectedGetLiveThreadTree = ThreadTree.getLiveTree.inject({
        ELECTRIC: ELECTRIC_TEST,
      });

      let result: ReturnType<typeof injectedGetLiveThreadTree>;
      $effect.root(() => {
        result = injectedGetLiveThreadTree(id);
      });

      const [unsubscribe, liveTree] = result._unsafeUnwrap({
        withStackTrace: true,
      });

      const derived = $derived(liveTree.state);
      describe("thread", () => {
        test("fractional index ordering", () => {
          expect(liveTree.state.child_threads[0]?.title).toBe("1");
          expect(liveTree.state.child_threads[1]?.title).toBe("2");
          expect(liveTree.state.child_threads[2]?.title).toBe("3");
          expect(liveTree.state.child_threads[3]?.title).toBe("4");
          expect(liveTree.state.child_threads[4]?.title).toBe("5");
        });

        test("exclude deleted thread", () => {
          expect(liveTree.state.child_threads[5]).toBeUndefined();
        });

        test("tree structuring", () => {
          expect(
            // @ts-expect-error thread nesting test
            liveTree.state.child_threads[0].child_threads[0].child_threads[0]
              .child_threads[0].child_threads[0].child_threads[0].id,
          ).toBeTruthy();
        });

        test("set parent", () => {
          expect(
            // @ts-expect-error thread nesting test
            liveTree.state.child_threads[0].child_threads[0].child_threads[0]
              .child_threads[0].child_threads[0].child_threads[0].parent.parent
              .parent.parent.parent.parent.id,
          ).toBe(id);
        });

        test("change field", () => {
          liveTree.state.title = "changed!";
          expect(liveTree.state.title).toBe("changed!");
          liveTree.state.child_threads[0].child_threads[0].parent.title =
            "changed!";
          expect(liveTree.state.child_threads[0].title).toBe("changed!");
          expect(derived.child_threads[0].title).toBe("changed!");
        });

        test("add element", async () => {
          liveTree.state.cards.push({
            id: "id",
            fractional_index: "f",
            content: "content",
          });
          // wait until effect run
          await new Promise((resolve) => setTimeout(resolve, 0));
          expect(
            liveTree.state.cards[liveTree.state.cards.length - 1].thread.id,
          ).toBe(id);
          liveTree.state.cards.pop();
        });

        test("move element", async () => {
          const card =
            liveTree.state?.child_threads[0]?.child_threads[0].cards.pop();
          liveTree.state?.child_threads[0].cards.push(card);
          // wait until effect run
          await new Promise((resolve) => setTimeout(resolve, 0));
          expect(
            liveTree.state?.child_threads[0].cards[
              liveTree.state?.cards.length - 1
            ]?.thread.parent.id,
          ).toBe(id);
        });

        test("unsubscribe", async () => {
          const resubscribe = unsubscribe();
          const newCard = await ELECTRIC_TEST.db.threads.create({
            data: {
              id: uuidv7(),
              title: "title",
              deleted: false,
              created_at: new Date(),
              updated_at: new Date(),
              fractional_index: "a0",
            },
          });
          await new Promise((resolve) => setTimeout(resolve, 100));
          expect(liveTree.state?.cards.length).toBe(5);
          resubscribe();
          await new Promise((resolve) => setTimeout(resolve, 100));
          expect(liveTree.state?.cards.length).toBe(6);
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
          expect(liveTree.state.cards[0]?.content).toBe("1");
          expect(liveTree.state.cards[1]?.content).toBe("2");
          expect(liveTree.state.cards[2]?.content).toBe("3");
          expect(liveTree.state.cards[3]?.content).toBe("4");
          expect(liveTree.state.cards[4]?.content).toBe("5");
        });

        test("exclude deleted card", () => {
          expect(liveTree.state.cards[5]).toBeUndefined();
        });

        test("tree structuring", () => {
          expect(
            // @ts-expect-error thread nesting test
            liveTree.state.child_threads[0].child_threads[0].child_threads[0]
              .child_threads[0].child_threads[0].child_threads[0]?.cards[0].id,
          ).toBeTruthy();
        });

        test("set parent", () => {
          expect(
            // @ts-expect-error thread nesting test
            liveTree.state.child_threads[0].child_threads[0].child_threads[0]
              .child_threads[0].child_threads[0].child_threads[0].cards[0]
              .thread.parent.parent.parent.parent.parent.parent.id,
          ).toBe(id);
        });
      });
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
          fractional_index: "",
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
        fractional_index: "",
        deleted: true,
        created_at: now,
        updated_at: now,
      },
    ],
  });
}
