import { ELECTRIC } from "$lib/DataAccess/electric";
import { execThrowable } from "$lib/Utils/neverthrow-utils";
import type { Result } from "neverthrow";
import type { ExtractArrayType } from "$lib/Utils/utilTypes";
import {
  getThreadTree,
  type ThreadTreeQueryResult,
} from "./queries/getThreadTree.sql";
import { depend } from "velona";
import { createLiveQuery } from "$lib/Utils/runesLiveQuery.svelte";
import { tick } from "svelte";

type Card = ExtractArrayType<ThreadTreeQueryResult["cards"]> & {
  thread: ThreadTree;
};

export type ThreadTree = Omit<
  ThreadTreeQueryResult,
  "child_threads" | "cards"
> & {
  cards: Card[];
  parent: ThreadTree | null;
  child_threads?: ThreadTree[];
};

export const ThreadTree = {
  getLiveTree: depend(
    { ELECTRIC },
    (
      { ELECTRIC },
      id: string,
    ): Result<[() => () => void, { state: ThreadTree | null }], Error> => {
      return execThrowable(() => {
        if (!ELECTRIC) throw new Error("electric has not initialized yet");
        const liveResult = createLiveQuery<{ json: string }[]>(
          ELECTRIC.notifier,
          ELECTRIC.db.liveRawQuery({
            sql: getThreadTree,
            args: [id],
          }),
        );

        // Using $state and $effect rather than $derived
        // to notify changes of the properties inside the result to dependants
        let state = $state<ThreadTree | null>(null);
        let isChangeFromLiveQuery: boolean;

        $effect(() => {
          if (!Array.isArray(liveResult.result)) {
            state = null;
          } else if (!liveResult.result[0]) {
            throw new Error("thread not found!");
          } else {
            state = JSON.parse(liveResult.result[0]["json"]);
          }
        });

        const removeHook = liveResult.addHook(async () => {
          // will run only after the first query result
          isChangeFromLiveQuery = true;
          await tick();
          const cache = localStorage.getItem(id);
          if (cache) {
            isChangeFromLiveQuery = false;
            state = JSON.parse(cache);
          } else {
            isChangeFromLiveQuery = false;
          }

          // will run from the second query result onward
          liveResult.addPreHook(() => {
            isChangeFromLiveQuery = true;
          });
          liveResult.addHook(async () => {
            await new Promise((resolve) => setTimeout(resolve, 0));
            localStorage.removeItem(id);
            isChangeFromLiveQuery = false;
          });

          removeHook();
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

        return [
          liveResult.unsubscribe,
          {
            get state() {
              return state;
            },
          },
        ];
      });
    },
  ),
};

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
        return null;
      },
      configurable: true,
    });
  }

  tree.cards.map((c) => {
    Object.defineProperty(c, "thread", {
      get: function () {
        return ref.deref();
      },
      configurable: true,
    });
    return c;
  });

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
