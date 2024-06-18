import { describe, expect } from "vitest";
import { Card } from "$lib/Models/Card";
import { testWithElectric } from "$lib/TestUtils/testWithElectric";

describe("card", async () => {
  testWithElectric("create card", async ({ electric }) => {
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    await injectedCreateCard();
    expect((await electric.db.cards.findMany()).length).toBe(1);
  });
  testWithElectric("update card", async ({ electric }) => {
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedUpdateCard = Card.update.inject({ ELECTRIC: electric });
    const card = await injectedCreateCard();
    await injectedUpdateCard({ ...card, content: "updated" });
    expect(
      (await electric.db.cards.findUnique({ where: { id: card.id } })).content,
    ).toBe("updated");
  });
});
