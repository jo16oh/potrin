import { expect } from "vitest";
import { testElectric } from "./testElectric";
import * as Y from "yjs";
import { Card } from "$lib/Models/Card";
import { Thread } from "$lib/Models/Thread";
import { uuidv7 } from "uuidv7";

testElectric("YDocMaterializer", async ({ electric }) => {
  const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
  const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
  const thread = await injectedCreateThread();
  const card = await injectedCreateCard({ thread_id: thread.id });

  const ydoc = new Y.Doc();

  ydoc.on("updateV2", (update) => {
		// @ts-expect-error to avoid error in test
		if (typeof process !== "undefined" && !electric.adapter.db.open) return;
    electric.db.card_ydoc_updates.create({
      data: {
        id: uuidv7(),
        card_id: card.id,
        data: update,
        checkpoint: false,
        created_at: new Date(),
      },
    });
  });

  const xml = ydoc.getXmlFragment();
  const element = new Y.XmlElement("div");
  element.insert(0, [new Y.XmlText("content")]);
  xml.insert(0, [element]);

  const updates = await electric.db.card_ydoc_updates.findMany();
  expect(updates.length).toBeTruthy();

	await new Promise(resolve => setTimeout(resolve, 100))
	const updatedCard = await electric.db.cards.findUnique({where: {id: card.id}})
	expect(updatedCard.content).toBe('<div>content</div>')
});
