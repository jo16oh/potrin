import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Cards } from "../../generated/client";
import type { Optional } from "utility-types";
import { depend } from "velona";
import type { ThreadTree } from "./ThreadTree.svelte";

export type Card = Optional<
  Cards,
  "deleted" | "created_at" | "updated_at" | "thread_id"
> & {
  thread?: ThreadTree;
};

export const Card = {
  create: depend(
    { ELECTRIC },
    async ({ ELECTRIC }, card?: Partial<Card>): Promise<Card> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      const now = new Date();
      return (await ELECTRIC.db.cards.create({
        data: {
          ...card,
          id: card?.id ? card.id : uuidv7(),
          thread_id: card?.thread_id ? card.thread_id : uuidv7(),
          fractional_index: card?.fractional_index
            ? card.fractional_index
            : "a0",
          content: card?.content ? card.content : "",
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
};
