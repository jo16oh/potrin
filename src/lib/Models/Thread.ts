import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Threads } from "../../generated/client";
import { depend } from "velona";
import type { Optional } from "utility-types";

export type Thread = Optional<
  Threads,
  "deleted" | "created_at" | "updated_at" | "parent_id"
>;

export const Thread = {
  create: depend(
    { ELECTRIC },
    async ({ ELECTRIC }, thread?: Partial<Thread>): Promise<Thread> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      const now = new Date();
      return (await ELECTRIC.db.threads.create({
        data: {
          ...thread,
          id: thread?.id ? thread.id : uuidv7(),
          parent_id: thread?.parent_id ? thread.parent_id : uuidv7(),
          fractional_index: thread?.fractional_index
            ? thread.fractional_index
            : "a0",
          title: thread?.title ? thread.title : "",
          created_at: now,
          updated_at: now,
          deleted: false,
        },
      })) as Thread;
    },
  ),
  update: depend(
    { ELECTRIC },
    async (
      { ELECTRIC },
      thread: Omit<Thread, "created_at" | "updated_at" | "deleted">,
    ): Promise<Thread> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      return (await ELECTRIC.db.threads.update({
        where: { id: thread.id },
        data: {
          ...thread,
          updated_at: new Date(),
        },
      })) as Thread;
    },
  ),
};
