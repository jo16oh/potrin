import { expect, describe, afterEach } from "vitest";
import { uuidv7 } from "uuidv7";
import { generateKeyBetween } from "fractional-indexing";
import { testElectric, testElectricSync } from "$lib/DataAccess/testElectric";
import { ThreadTree } from "./ThreadTree.svelte";
import type { ElectricClient } from "electric-sql/client/model";
import type { schema } from "../../generated/client";
import { Thread } from "./Thread";
import { Card } from "./Card";
import { sql } from "$lib/Utils/utils";
import { exec } from "node:child_process";
import util from "node:util";

interface TestThreadTree {
  liveTree: ReturnType<typeof ThreadTree.liveFullTree>;
}

const testThreadTree = testElectric.extend<TestThreadTree>({
  liveTree: async ({ electric }, use) => {
    const id = await createTree(electric);
    const injectedGetLiveTree = ThreadTree.liveFullTree.inject({
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

const testThreadTreeWithCache = testElectric.extend<TestThreadTree>({
  liveTree: async ({ electric }, use) => {
    const id = await createTree(electric);
    const injectedGetLiveTree = ThreadTree.liveFullTree.inject({
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

describe("ThreadTree", () => {
  describe("thread", () => {
    testThreadTree("fractional index ordering", ({ liveTree }) => {
      expect(liveTree?.state?.child_threads?.[0]?.title).toBe("1");
      expect(liveTree?.state?.child_threads?.[1]?.title).toBe("2");
      expect(liveTree?.state?.child_threads?.[2]?.title).toBe("3");
      expect(liveTree?.state?.child_threads?.[3]?.title).toBe("4");
      expect(liveTree?.state?.child_threads?.[4]?.title).toBe("5");
    });

    testThreadTree("exclude deleted thread", ({ liveTree }) => {
      expect(liveTree.state?.child_threads?.[5]).toBeUndefined();
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
          .child_threads[0].child_threads[0].child_threads[0].parent.parent
          .parent.parent.parent.parent.id,
      ).toBe(liveTree.state.id);
    });

    testThreadTree("change field", ({ liveTree }) => {
      liveTree.state.title = "changed!";
      expect(liveTree.state.title).toBe("changed!");
      liveTree.state.child_threads[0].child_threads[0].parent.title =
        "changed!";
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
      const card =
        liveTree.state?.child_threads[0]?.child_threads[0].cards.pop();
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
            last_materialized: "",
            thread_id: liveTree.state.id,
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
        where: { thread_id: liveTree.state.id },
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

  describe.skip("cache", () => {
    testThreadTreeWithCache(
      "initial state of the liveTree to be cache",
      async ({ liveTree }) => {
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
        await new Promise((resolve) => setTimeout(resolve, 0));
        expect(JSON.parse(localStorage.getItem(liveTree.state.id))).toBe(null);
      },
    );

    testThreadTreeWithCache(
      "cache should be updated along with the state change",
      async ({ liveTree }) => {
        liveTree.state.title = "3";
        await new Promise((resolve) => setTimeout(resolve, 100));
        expect(JSON.parse(localStorage.getItem(liveTree.state.id))?.title).toBe(
          "3",
        );
      },
    );
  });

  describe("sync", () => {
    testElectricSync(
      "When e1 deletes the root thread and e2 adds children to it while they are offline, all rows should eventually be deleted",
      async ({ e1, e2, token }) => {
        const injectedCreateThread1 = Thread.create.inject({
          ELECTRIC: e1,
        });
        const injectedCreateThread2 = Thread.create.inject({
          ELECTRIC: e2,
        });
        const injectedCreateCard2 = Card.create.inject({ ELECTRIC: e2 });
        const injectedDeleteThread = Thread.deletePhysical.inject({
          ELECTRIC: e1,
        });

        // connection
        expect(e1.isConnected).toBeTruthy();
        expect(e2.isConnected).toBeTruthy();

        // initial cleanup
        const initialState1 = await e1.db.threads.findMany();
        const initialState2 = await e2.db.threads.findMany();
        expect((await e1.db.cards.findMany()).length).toBe(0);
        expect((await e2.db.cards.findMany()).length).toBe(0);
        expect(initialState1.length).toBe(0);
        expect(initialState2.length).toBe(0);

        // sync ( e1 → e2 )
        const { id } = await injectedCreateThread1();
        await new Promise((resolve) => setTimeout(resolve, 1000));
        const res = await e2.db.threads.findUnique({
          where: { id: id },
        });
        expect(res.id).toBe(id);

        // disconnect and deletes the root
        e1.disconnect();
        e2.disconnect();
        await new Promise((resolve) => setTimeout(resolve, 1000));

        await injectedDeleteThread(id);

        const child = await injectedCreateThread2({ parent_id: id });
        const grandChild = await injectedCreateThread2({ parent_id: child.id });
        await new Promise((resolve) => setTimeout(resolve, 1000));
        await injectedCreateCard2({ thread_id: child.id });
        await injectedCreateCard2({ thread_id: grandChild.id });
        await new Promise((resolve) => setTimeout(resolve, 1000));

        // resync
        await e1.connect(token);
        await e2.connect(token);
        const shape1_2 = await e1.db.threads.sync();
        const shape2_2 = await e2.db.threads.sync();
        const shape3_2 = await e1.db.cards.sync();
        const shape4_2 = await e2.db.cards.sync();
        await Promise.all([
          shape1_2.synced,
          shape2_2.synced,
          shape3_2.synced,
          shape4_2.synced,
        ]);
        await new Promise((resolve) => setTimeout(resolve, 1000));

        expect((await e1.db.threads.findMany()).length).toBe(0);
        expect((await e2.db.threads.findMany()).length).toBe(0);
        expect((await e1.db.cards.findMany()).length).toBe(0);
        expect((await e2.db.cards.findMany()).length).toBe(0);
      },
      30000,
    );
  });

  testElectric("liveNode", async ({ electric }) => {
    const injectedLiveNode = ThreadTree.liveNode.inject({ ELECTRIC: electric });
    const root = await createTree(electric);
    await new Promise((resolve) => setTimeout(resolve, 0));
    let liveNode;
    $effect.root(() => {
      liveNode = injectedLiveNode(root);
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(liveNode?.child_threads).toBeUndefined();
  });

  testThreadTree("livePartialTree", async ({ electric, liveTree }) => {
    const injectedLivePartialTree = ThreadTree.livePartialTree.inject({
      ELECTRIC: electric,
    });

    liveTree.unsubscribe();
    localStorage.clear();

    let livePartialTree;
    $effect.root(() => {
      livePartialTree = injectedLivePartialTree(liveTree.state.id);
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(
      livePartialTree.state.child_threads[0].child_threads[0]?.cards,
    ).toBeUndefined();
  });

  testThreadTree("breadcrumbs", async ({ electric, liveTree }) => {
    const injectedLivePartialTree = ThreadTree.livePartialTree.inject({
      ELECTRIC: electric,
    });

    const id =
      liveTree.state?.child_threads[0]?.child_threads[0]?.child_threads[0].id;
    liveTree.unsubscribe();
    localStorage.clear();

    let livePartialTree;
    $effect.root(() => {
      livePartialTree = injectedLivePartialTree(id);
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(livePartialTree.state.breadcrumbs[0].id).toBe(liveTree.state.id);
  });
});

const createTree = async (
  electric: ElectricClient<typeof schema>,
  depth: number = 0,
  parent_id?: string,
) => {
  const currentID = uuidv7();
  if (depth > 6) return currentID;

  const first = generateKeyBetween(null, null);
  const third = generateKeyBetween(first, null);
  const second = generateKeyBetween(first, third);
  const fourth = third;
  const fifth = third;
  const now = new Date();

  if (parent_id) {
    await electric.db.threads.createMany({
      data: [
        {
          id: currentID,
          title: "1",
          parent_id: parent_id,
          fractional_index: first,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "2",
          parent_id: parent_id,
          fractional_index: second,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "3",
          parent_id: parent_id,
          fractional_index: third,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "4",
          parent_id: parent_id,
          fractional_index: fourth,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "5",
          parent_id: parent_id,
          fractional_index: fifth,
          deleted: false,
          created_at: now,
          updated_at: now,
        },
        {
          id: uuidv7(),
          title: "deleted",
          parent_id: parent_id,
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
  thread_id: string,
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
        id: thread_id,
        content: "1",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: first,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "2",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: second,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "3",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: third,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "4",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: fourth,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "5",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: fifth,
        deleted: false,
        created_at: now,
        updated_at: now,
      },
      {
        id: uuidv7(),
        content: "deleted",
        last_materialized: "",
        thread_id: thread_id,
        fractional_index: "",
        deleted: true,
        created_at: now,
        updated_at: now,
      },
    ],
  });
}
