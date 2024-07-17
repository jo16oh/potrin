import { describe, expect } from "vitest";
import { Card } from "$lib/Models/Card";
import { testElectric } from "$lib/DataAccess/testElectric";
import { Thread } from "./Thread";
import { uuidv7 } from "uuidv7";
import * as Y from "yjs";
import { YDocMatelializer } from "$lib/DataAccess/YDocMaterializer";

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

  testElectric("getElectrcYDoc", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedGetCardYDoc = Card.getElectricYDoc.inject({
      ELECTRIC: electric,
    });

    const thread = await injectedCreateThread();
    const card = await injectedCreateCard({ thread_id: thread.id });
    const ydoc = await injectedGetCardYDoc(card.id);
    const xml = ydoc.getXmlFragment("prosemirror");

    xml.insert(0, [new Y.XmlElement("div")]);

    await new Promise((resolve) => setTimeout(resolve, 100));
    const materialized = await electric.db.cards.findUnique({
      where: { id: card.id },
    });

    expect(materialized.content).toBe(xml.toString());
  });

  testElectric("checkpoint", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
    const injectedGetCardYDoc = Card.getElectricYDoc.inject({
      ELECTRIC: electric,
    });
    const injectedCheckpoint = Card.checkpoint.inject({
      ELECTRIC: electric,
    });

    const thread = await injectedCreateThread();
    const card = await injectedCreateCard({ thread_id: thread.id });

    {
      const ydoc = await injectedGetCardYDoc(card.id);
      const xml = ydoc.getXmlFragment("prosemirror");
      xml.insert(0, [new Y.XmlElement("div")]);
      xml.insert(0, [new Y.XmlElement("div")]);
      await injectedCheckpoint(card.id);
      xml.insert(0, [new Y.XmlElement("div")]);
      xml.insert(0, [new Y.XmlElement("div")]);
      await injectedCheckpoint(card.id);
    }

    {
      const updates = await electric.db.card_ydoc_updates.findMany();
      expect(updates.length).toBe(4);
    }

    {
      const checkpoints = await electric.db.card_ydoc_updates.findMany({
        where: { checkpoint: true },
        orderBy: { created_at: "desc" },
      });
      expect(checkpoints.length).toBe(2);
    }

    {
      await YDocMatelializer.mergeCardUpdates(card.id, 1000, electric);
      const updates = await electric.db.card_ydoc_updates.findMany();
      expect(updates.length).toBe(2);

      const ydoc = await injectedGetCardYDoc(card.id);
      const xml = ydoc.getXmlFragment("prosemirror");

      expect(xml.toString()).toBe(
        "<div></div><div></div><div></div><div></div>",
      );
    }
  });
});
