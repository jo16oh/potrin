import { describe, expect } from "vitest";
import { Card } from "$lib/Models/Card";
import { uuidv7 } from "uuidv7";
import { testWithElectric } from "$lib/TestUtils/testWithElectric";

describe("card", async () => {
  const id = uuidv7();
  testWithElectric("create card", async ({ electric }) => {
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    await injectedCreateCard({ id: id });
    expect((await electric.db.cards.findMany()).length).toBe(1);
  });
  testWithElectric("update card", async ({ electric }) => {
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedUpdateCard = Card.update.inject({ ELECTRIC: electric });
    await injectedCreateCard({ id: id });
    await injectedUpdateCard(id, { content: "updated" });
    expect(
      (await electric.db.cards.findUnique({ where: { id: id } })).content,
    ).toBe("updated");
  });
});
