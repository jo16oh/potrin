import { ELECTRIC } from "$lib/DataAccess/electric";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";
import type { ResultAsync } from "neverthrow";
import { query, type ThreadTree } from "./queries/getThreadTree.sql";
import { depend } from "velona";

export const getLiveThreadTree = depend(
  { ELECTRIC },
  ({ ELECTRIC }, id: string): ResultAsync<ThreadTree, Error> => {
    return execAsyncThrowable(async () => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      const result = await ELECTRIC.db.rawQuery({ sql: query, args: [id] });
      const parsed = JSON.parse(result[0].json) as ThreadTree;
      return parsed;
    });
  },
);
