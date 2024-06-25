import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Threads } from "../../generated/client";
import { depend } from "velona";
import type { Optional } from "utility-types";
import { sql } from "$lib/Utils/utils";

export type Thread = Optional<
  Threads,
  "deleted" | "created_at" | "updated_at" | "parent_id"
>;

export const Thread = {
  create: depend(
    { ELECTRIC },
    async ({ ELECTRIC }, thread?: Partial<Thread>): Promise<Thread> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");

      if (thread?.parent_id) {
        const parent = await ELECTRIC.db.threads.findUnique({
          where: { id: thread.parent_id },
        });
        if (!parent) throw new Error("parent not found!");
      }

      const now = new Date();
      return (await ELECTRIC.db.threads.create({
        data: {
          ...thread,
          id: thread?.id || uuidv7(),
          parent_id: thread?.parent_id || null,
          fractional_index: thread?.fractional_index || "a0",
          title: thread?.title || "",
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

  deleteLogical: depend({ ELECTRIC }, async ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return ELECTRIC.db.threads.update({
      where: { id: id },
      data: { deleted: true },
    });
  }),

  deletePhysical: depend({ ELECTRIC }, async ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return ELECTRIC.adapter.runInTransaction(
      {
        sql: sql`
					CREATE TEMPORARY TABLE IF NOT EXISTS temp_thread_tree (id string);
				`,
      },
      {
        sql: sql`
					WITH RECURSIVE thread_tree AS (
						SELECT threads.id, threads.parent_id 
						FROM threads
						WHERE id = ?
						UNION ALL
						SELECT child.id, child.parent_id 
						FROM thread_tree AS parent
						JOIN threads AS child ON parent.id = child.parent_id
					)
					INSERT INTO temp_thread_tree (id)
					SELECT id FROM thread_tree;
				`,
        args: [id],
      },
      {
        sql: sql`
					DELETE FROM threads WHERE id IN (SELECT id FROM temp_thread_tree);
				`,
      },
      {
        sql: sql`
					DELETE FROM cards WHERE thread_id IN (SELECT id FROM temp_thread_tree);
				`,
      },
      {
        sql: sql`
					DROP TABLE temp_thread_tree;
				`,
      },
    );
  }),
};
