import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Threads as Thread } from "../../generated/client";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";
import { ResultAsync } from "neverthrow";
import { depend } from "velona";

export const createThread = depend(
  { ELECTRIC },
  ({ ELECTRIC }, thread: Partial<Thread>): ResultAsync<Thread, Error> => {
    return execAsyncThrowable(async () => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      return (await ELECTRIC.db.threads.create({
        data: {
          ...thread,
          id: thread.id ? thread.id : uuidv7(),
        },
      })) as Thread;
    });
  },
);
