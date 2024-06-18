import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Cards } from "../../generated/client";
import { depend } from "velona";

export type Card = Cards;

export const Card = {
  create: depend(
    { ELECTRIC },
    async ({ ELECTRIC }, card: Partial<Card>): Promise<Card> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      const now = new Date();
      return (await ELECTRIC.db.cards.create({
        data: {
          ...card,
          id: card.id ? card.id : uuidv7(),
          thread: card.thread ? card.thread : uuidv7(),
          fractional_index: card.fractional_index
            ? card.fractional_index
            : "a0",
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
      id: string,
      card: Omit<Partial<Card>, "id" | "created_at" | "updated_at">,
    ): Promise<Card> => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      return (await ELECTRIC.db.cards.update({
        where: { id: id },
        data: {
          updated_at: new Date(),
          ...card,
        },
      })) as Card;
    },
  ),
};
