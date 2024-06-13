import { ELECTRIC } from "$lib/DataAccess/electric";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";
import type { ResultAsync } from "neverthrow";
import {
  getThreadTree,
  type ThreadTreeQueryResult,
} from "./queries/getThreadTree.sql";
import { depend } from "velona";

export type ThreadTree = Omit<
  ThreadTreeQueryResult,
  "child_threads" | "cards"
> & {
  cards: Omit<ThreadTreeQueryResult["cards"], "thread"> &
    {
      thread: ThreadTree;
    }[];
  parent: ThreadTree;
  child_threads?: ThreadTree[];
};

export const ThreadTree = {
  getLiveTree: depend(
    { ELECTRIC },
    ({ ELECTRIC }, id: string): ResultAsync<ThreadTree, Error> => {
      return execAsyncThrowable(async () => {
        if (!ELECTRIC) throw new Error("electric has not initialized yet");
        const result = await ELECTRIC.db.rawQuery({
          sql: getThreadTree,
          args: [id],
        });
        if (!Array.isArray(result)) throw new Error("query failed!");
        if (!result[0]) throw new Error("query failed!");
        if (typeof result[0]["json"] !== "string")
          throw new Error("query failed!");
        const parsed = JSON.parse(result[0]["json"]) as ThreadTreeQueryResult;
        return setParent(parsed);
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
    });
  }

  tree.cards.map((c) => {
    if (Object.prototype.hasOwnProperty.call(c, "thread")) return c;
    Object.defineProperty(c, "thread", {
      get: function () {
        return ref.deref();
      },
    });
    return c;
  });

  if (tree?.child_threads) {
    tree.child_threads.map((c) => {
      if (Object.prototype.hasOwnProperty.call(c, "parent")) return c;
      Object.defineProperty(c, "parent", {
        get: function () {
          return ref.deref();
        },
      });
      c = setParent(c, false);
      return c;
    });
  }

  return tree as ThreadTree;
}
