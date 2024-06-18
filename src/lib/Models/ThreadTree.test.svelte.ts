import { expect, describe, afterEach } from "vitest";
import { uuidv7 } from "uuidv7";
import { generateKeyBetween } from "fractional-indexing";
import { testWithElectric } from "$lib/TestUtils/testWithElectric";
import { ThreadTree } from "./ThreadTree.svelte";
import type { ElectricClient } from "electric-sql/client/model";
import type { schema } from "../../generated/client";

interface TestThreadTree {
  liveTree: ReturnType<typeof ThreadTree.getLiveTree>;
}

const testThreadTree = testWithElectric.extend<TestThreadTree>({
  liveTree: async ({ electric }, use) => {
    const id = await createTree(electric, 0);
    const injectedGetLiveTree = ThreadTree.getLiveTree.inject({
      ELECTRIC: electric,
    });

    let liveTree: ReturnType<typeof injectedGetLiveTree>;
    const cleanup = $effect.root(() => {
      liveTree = injectedGetLiveTree(id);
    });
    await new Promise((resolve) => setTimeout(resolve, 0));

    use(liveTree);

    afterEach(() => {
      cleanup();
    });
  },
});

const testThreadTreeWithCache = testWithElectric.extend<TestThreadTree>({
  liveTree: async ({ electric }, use) => {
    const id = await createTree(electric, 0);
    const injectedGetLiveTree = ThreadTree.getLiveTree.inject({
      ELECTRIC: electric,
    });

    const cache = JSON.stringify({ id: id, title: "cache", cards: [] });
    localStorage.setItem(id, cache);

    let liveTree: ReturnType<typeof injectedGetLiveTree>;
    const cleanup = $effect.root(() => {
      liveTree = injectedGetLiveTree(id);
    });
    await new Promise((resolve) => setTimeout(resolve, 0));

    use(liveTree);

    afterEach(() => {
      cleanup();
      localStorage.clear();
    });
  },
});

describe("thread", () => {
  testThreadTree("fractional index ordering", ({ liveTree }) => {
    expect(liveTree.state.child_threads[0]?.title).toBe("1");
    expect(liveTree.state.child_threads[1]?.title).toBe("2");
    expect(liveTree.state.child_threads[2]?.title).toBe("3");
    expect(liveTree.state.child_threads[3]?.title).toBe("4");
    expect(liveTree.state.child_threads[4]?.title).toBe("5");
  });

  testThreadTree("exclude deleted thread", ({ liveTree }) => {
    expect(liveTree.state.child_threads[5]).toBeUndefined();
  });

  testThreadTree("tree structuring", ({ liveTree }) => {
    expect(
      // @ts-expect-error thread nesting test
      liveTree.state.child_threads[0].child_threads[0].child_threads[0]
        .child_threads[0].child_threads[0].child_threads[0].id,
    ).toBeTruthy();
  });

  testThreadTree("set parent", ({ liveTree }) => {
    expect(
      // @ts-expect-error thread nesting test
      liveTree.state.child_threads[0].child_threads[0].child_threads[0]
        .child_threads[0].child_threads[0].child_threads[0].parent.parent.parent
        .parent.parent.parent.id,
    ).toBe(liveTree.state.id);
  });

  testThreadTree("change field", ({ liveTree }) => {
    liveTree.state.title = "changed!";
    expect(liveTree.state.title).toBe("changed!");
    liveTree.state.child_threads[0].child_threads[0].parent.title = "changed!";
    expect(liveTree.state.child_threads[0].title).toBe("changed!");
  });

  testThreadTree("add element", async ({ liveTree }) => {
    liveTree.state.cards.push({
      id: "id",
      fractional_index: "f",
      content: "content",
    });
    // wait until effect run
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(
      liveTree.state.cards[liveTree.state.cards.length - 1].thread.id,
    ).toBe(liveTree.state.id);
  });

  testThreadTree("move element", async ({ liveTree }) => {
    const card = liveTree.state?.child_threads[0]?.child_threads[0].cards.pop();
    liveTree.state?.child_threads[0].cards.push(card);
    // wait until effect run
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(
      liveTree.state?.child_threads[0].cards[liveTree.state?.cards.length - 1]
        ?.thread.parent.id,
    ).toBe(liveTree.state.id);
  });

  testThreadTree(
    "unsubscribe / resubscribe",
    async ({ liveTree, electric }) => {
      const resubscribe = liveTree.unsubscribe();
      await electric.db.cards.create({
        data: {
          id: uuidv7(),
          content: "title",
          thread: liveTree.state.id,
          deleted: false,
          created_at: new Date(),
          updated_at: new Date(),
          fractional_index: "a0",
        },
      });
      expect(liveTree.state?.cards.length).toBe(5);
      resubscribe();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(liveTree.state?.cards.length).toBe(6);
    },
  );
});

describe("card", () => {
  testThreadTree("card creation", async ({ liveTree, electric }) => {
    const result = await electric.db.cards.findMany({
      where: { thread: liveTree.state.id },
    });
    expect(result.length).toBe(6);
  });

  testThreadTree("fractional index ordering", ({ liveTree }) => {
    expect(liveTree.state.cards[0]?.content).toBe("1");
    expect(liveTree.state.cards[1]?.content).toBe("2");
    expect(liveTree.state.cards[2]?.content).toBe("3");
    expect(liveTree.state.cards[3]?.content).toBe("4");
    expect(liveTree.state.cards[4]?.content).toBe("5");
  });

  testThreadTree("exclude deleted card", ({ liveTree }) => {
    expect(liveTree.state.cards[5]).toBeUndefined();
  });

  testThreadTree("tree structuring", ({ liveTree }) => {
    expect(
      // @ts-expect-error thread nesting test
      liveTree.state.child_threads[0].child_threads[0].child_threads[0]
        .child_threads[0].child_threads[0].child_threads[0]?.cards[0].id,
    ).toBeTruthy();
  });

  testThreadTree("set parent", ({ liveTree }) => {
    expect(
      // @ts-expect-error thread nesting test
      liveTree.state.child_threads[0].child_threads[0].child_threads[0]
        .child_threads[0].child_threads[0].child_threads[0].cards[0].thread
        .parent.parent.parent.parent.parent.parent.id,
    ).toBe(liveTree.state.id);
  });
});

describe("cache", () => {
  testThreadTreeWithCache(
    "initial state of the liveTree to be cache",
    async ({ liveTree }) => {
      // この最初のsetTimeoutがないとテストが失敗する、なぜ？
      // beforeEachやtest.extend内でawaitしてもだめ
      expect(liveTree.state?.title).toBe("cache");
    },
  );

  testThreadTreeWithCache(
    "unsubscribe → resubscribe",
    async ({ liveTree, electric }) => {
      const subscribe = liveTree.unsubscribe();
      await electric.db.threads.update({
        where: { id: liveTree.state.id },
        data: { title: "updated" },
      });
      expect(liveTree.state?.title).toBe("cache");

      subscribe();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(liveTree.state?.title).toBe("updated");
    },
  );

  testThreadTreeWithCache(
    "cache should be removed when received db updates",
    async ({ electric, liveTree }) => {
      await electric.db.threads.update({
        where: { id: liveTree.state.id },
        data: { title: "updated" },
      });
      await new Promise((resolve) => setTimeout(resolve, 100));
      expect(JSON.parse(localStorage.getItem(liveTree.state.id))).toBe(null);
    },
  );

  testThreadTreeWithCache(
    " cache should be updated along with the state change",
    async ({ liveTree }) => {
      liveTree.state.title = "3";
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(JSON.parse(localStorage.getItem(liveTree.state.id))?.title).toBe(
        "3",
      );
    },
  );
});

const createTree = async (
  electric: ElectricClient<typeof schema>,
  depth: number,
  parent_thread?: string,
) => {
  const currentID = uuidv7();
  if (depth > 6) return currentID;

  const first = generateKeyBetween(null, null);
  const third = generateKeyBetween(first, null);
  const second = generateKeyBetween(first, third);
  const fourth = third;
  const fifth = third;
  const now = new Date();

  if (parent_thread) {
    await electric.db.threads.createMany({
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

    await createTree(electric, depth + 1, currentID);
  } else {
    await electric.db.threads.create({
      data: {
        id: currentID,
        title: "root",
        fractional_index: first,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
    });
    await createTree(electric, depth + 1, currentID);
  }

  await createCard(electric, currentID);

  return currentID;
};

async function createCard(
  electric: ElectricClient<typeof schema>,
  thread: string,
) {
  const first = generateKeyBetween(null, null);
  const third = generateKeyBetween(first, null);
  const second = generateKeyBetween(first, third);
  const fourth = third;
  const fifth = third;
  const now = new Date();
  await electric.db.cards.createMany({
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
