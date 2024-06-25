import { depend } from "velona";
import { ELECTRIC } from "$lib/DataAccess/electric";
import type { ElectricClient } from "electric-sql/client/model";
import type { schema } from "../../generated/client";
import { createLiveQuery } from "$lib/Utils/runesLiveQuery.svelte";
import { Card } from "./Card";
import { Thread } from "./Thread";
import { getFullTree } from "./queries/getFullTree.sql";
import { getPartialTree } from "./queries/getPartialTree.sql";
import { getNode } from "./queries/getNode.sql";
import type {
  ThreadTreeQueryResult,
  ThreadTreeQueryRawResult,
} from "./queries/ThreadTreeQueryTypes";

export type ThreadTree = Thread & {
  cards: Card[];
  parent?: ThreadTree;
  child_threads?: ThreadTree[];
  breadcrumbs?: {
    id: string;
    title: string;
  };
};

export const ThreadTree = {
  createNode: async (tree?: {
    thread?: Partial<ThreadTree>;
    card?: Partial<Card>;
  }): Promise<ThreadTree> => {
    const thread = await Thread.create(tree?.thread);
    const card = await Card.create({ ...tree?.card, thread_id: thread.id });
    return {
      ...thread,
      cards: [card],
    };
  },

  liveFullTree: depend({ ELECTRIC }, ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return getLiveTree(ELECTRIC, id, getFullTree);
  }),

  livePartialTree: depend({ ELECTRIC }, ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return getLiveTree(ELECTRIC, id, getPartialTree);
  }),

  liveNode: depend({ ELECTRIC }, ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return getLiveTree(ELECTRIC, id, getNode);
  }),
};

function getLiveTree(
  electric: ElectricClient<typeof schema>,
  id: string,
  query: string,
): { state: ThreadTree | undefined; unsubscribe: () => () => void } {
  const liveResult = createLiveQuery<ThreadTreeQueryRawResult>(
    electric.notifier,
    electric.db.liveRawQuery({
      sql: query,
      args: [id],
    }),
  );

  // Using $state and $effect rather than $derived
  // to notify changes of the properties inside the result to dependants
  let state = $state<ThreadTree | undefined>(undefined);
  let isChangeFromLiveQuery: boolean;

  $effect(() => {
    if (!Array.isArray(liveResult.result)) {
      state = undefined;
    } else if (!liveResult.result[0]) {
      state = undefined;
      throw new Error("thread not found!");
    } else {
      state = JSON.parse(liveResult.result[0]["json"]) as ThreadTreeQueryResult;
    }
  });

  const removeHook = liveResult.addHook(async () => {
    removeHook();

    // will run only after the first query result
    isChangeFromLiveQuery = true;
    await new Promise((resolve) => resolve(null));
    const cache = localStorage.getItem(id);
    if (cache) {
      state = JSON.parse(cache) as ThreadTreeQueryResult;
    }
    isChangeFromLiveQuery = false;

    // will run from the second query result onward
    liveResult.addPreHook(() => {
      isChangeFromLiveQuery = true;
    });
    liveResult.addHook(async () => {
      localStorage.removeItem(id);

      // ensure flag will be set after all effects run
      await new Promise((resolve) => setTimeout(resolve, 0));
      isChangeFromLiveQuery = false;
    });
  });

  $effect(() => {
    if (!state) return;
    state = setParent(state);

    // if you put this line into if block below,
    // svelte won't be able to react to changes of the properties inside state
    const json = JSON.stringify(state);

    if (!isChangeFromLiveQuery) {
      localStorage.setItem(id, json);
    }
  });

  return {
    get state() {
      return state;
    },
    unsubscribe: liveResult.unsubscribe,
  };
}

// To set the reference correctly, the variable you pass to the function and the variable that gets the return value must be same, like this:
// let tree = setParent(tree)
function setParent(
  tree: ThreadTreeQueryResult | ThreadTree,
  root: boolean = true,
): ThreadTree {
  const ref = new WeakRef(tree);

  if (root) {
    Object.defineProperty(tree, "parent", {
      get: function () {
        return undefined;
      },
      configurable: true,
    });
  }

  if (tree?.cards) {
    tree.cards.map((c) => {
      Object.defineProperty(c, "thread", {
        get: function () {
          return ref.deref();
        },
        configurable: true,
      });
      return c;
    });
  }

  if (tree?.child_threads) {
    tree.child_threads.map((c) => {
      Object.defineProperty(c, "parent", {
        get: function () {
          return ref.deref();
        },
        configurable: true,
      });
      c = setParent(c, false);
      return c;
    });
  }

  return tree as ThreadTree;
}
