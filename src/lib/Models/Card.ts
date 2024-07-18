import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Cards } from "../../generated/client";
import type { Optional } from "utility-types";
import type { ThreadTree } from "./ThreadTree.svelte";
import { depend } from "velona";

export type Card = Optional<
  Cards,
  "deleted" | "created_at" | "updated_at" | "thread_id"
> & {
  thread?: ThreadTree;
};

type CardInput = Omit<
  Optional<Card, "id" | "fractional_index" | "content">,
  | "thread_id"
  | "created_at"
  | "updated_at"
  | "deleted"
  | "ydoc_id"
  | "author"
  | "pot_id"
  | "last_materialized_hash"
> & { thread_id: string };

export const Card = {
  create: depend(
    { ELECTRIC },
    async ({ ELECTRIC }, card: CardInput): Promise<Card> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");

      const thread = await ELECTRIC.db.threads.findUnique({
        where: { id: card.thread_id },
      });
      if (!thread) throw new Error("thread doesn't exist");

      const now = new Date();
      return (await ELECTRIC.db.cards.create({
        data: {
          ...card,
          id: card?.id || uuidv7(),
          thread_id: card.thread_id,
          fractional_index: card?.fractional_index || "a0",
          content: card.content || "",
          last_materialized_hash: "",
          created_at: now,
          updated_at: now,
          deleted: false,
        },
      })) as Card;
    },
  ),

  update: depend(
    { ELECTRIC },
    async (
      { ELECTRIC },
      card: Omit<Card, "created_at" | "updated_at" | "deleted">,
    ): Promise<Card> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      return (await ELECTRIC.db.cards.update({
        where: { id: card.id },
        data: {
          updated_at: new Date(),
          ...card,
        },
      })) as Card;
    },
  ),

  deleteLogical: depend({ ELECTRIC }, async ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return await ELECTRIC.db.cards.update({
      where: { id: id },
      data: { deleted: true },
    });
  }),

  deletePhysical: depend({ ELECTRIC }, async ({ ELECTRIC }, id: string) => {
    if (!ELECTRIC) throw new Error("electric has not initialized yet");
    return await ELECTRIC.db.cards.delete({
      where: { id: id },
    });
  }),
} as const;
