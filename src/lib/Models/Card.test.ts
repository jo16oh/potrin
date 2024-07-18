import { describe, expect } from "vitest";
import { Card } from "$lib/Models/Card";
import { testElectric } from "$lib/DataAccess/testElectric";
import { Thread } from "./Thread";
import { uuidv7 } from "uuidv7";

describe("card", () => {
  testElectric("create card", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const thread = await injectedCreateThread();
    await injectedCreateCard({ thread_id: thread.id });
    expect((await electric.db.cards.findMany()).length).toBe(1);
  });

  testElectric("update card", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedUpdateCard = Card.update.inject({ ELECTRIC: electric });

    const thread = await injectedCreateThread();
    const card = await injectedCreateCard({ thread_id: thread.id });
    await injectedUpdateCard({ ...card, content: "updated" });
    expect(
      (await electric.db.cards.findUnique({ where: { id: card.id } }))?.[
        "content"
      ],
    ).toBe("updated");
  });

  testElectric("delete logical", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedDeleteCardLogical = Card.deleteLogical.inject({
      ELECTRIC: electric,
    });
    const thread = await injectedCreateThread();
    const card = await injectedCreateCard({ thread_id: thread.id });
    await injectedDeleteCardLogical(card.id);
    const deleted = await electric.db.cards.findUnique({
      where: { id: card.id },
    });
    expect(deleted.deleted).toBeTruthy();
  });

  testElectric("delete physical", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedDeleteCardPhysical = Card.deletePhysical.inject({
      ELECTRIC: electric,
    });
    const thread = await injectedCreateThread();
    const card = await injectedCreateCard({ thread_id: thread.id });
    await injectedDeleteCardPhysical(card.id);
    const deleted = await electric.db.cards.findUnique({
      where: { id: card.id },
    });
    expect(deleted).toBeFalsy();
  });

  testElectric("fails if the thread doesn't exists", async ({ electric }) => {
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    expect(injectedCreateCard({ thread_id: uuidv7() })).rejects.toThrow();
  });
});
