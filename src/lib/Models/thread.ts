import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Threads } from "../../generated/client";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";
import { ResultAsync } from "neverthrow";
import { depend } from "velona";

export type Thread = Threads;

export const Thread = {
  create: depend(
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
  ),
  update: depend(
    { ELECTRIC },
    (
      { ELECTRIC },
      id: string,
      thread: Omit<Partial<Thread>, "id" | "created_at" | "updated_at">,
    ): ResultAsync<Thread, Error> => {
      return execAsyncThrowable(async () => {
        if (!ELECTRIC) throw new Error("electric has not initialized yet");
        return (await ELECTRIC.db.threads.update({
          where: { id: id },
          data: {
            ...thread,
            updated_at: new Date(),
          },
        })) as Thread;
      });
    },
  ),
};
