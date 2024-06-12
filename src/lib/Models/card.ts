import { ELECTRIC } from "$lib/DataAccess/electric";
import { uuidv7 } from "uuidv7";
import type { Cards as Card } from "../../generated/client";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";
import { ResultAsync } from "neverthrow";
import { depend } from "velona";

export const createCard = depend(
  { ELECTRIC },
  ({ ELECTRIC }, card: Partial<Card>): ResultAsync<Card, Error> => {
    return execAsyncThrowable(async () => {
      if (!ELECTRIC) throw new Error("electric has not initialized yet");
      return (await ELECTRIC.db.cards.create({
        data: {
          ...card,
          id: card.id ? card.id : uuidv7(),
        },
      })) as Card;
    });
  },
);
