import { test, expect } from "vitest";

test.skip("CardYDoc", () => {
  expect(true).toBeTruthy()
})
// import { testElectric } from "$lib/DataAccess/testElectric";
// import { Thread } from "./Thread";
// import { Card } from "./Card";
// import { YDocMatelializer } from "$lib/DataAccess/YDocMaterializer";
// import { CardYDoc } from "./CardYDoc";
// import * as Y from "yjs";
//
// testElectric("CardYDoc", async ({ electric }) => {
//   const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
//   const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
//   const injectedGetCardYDoc = CardYDoc.init.inject({
//     ELECTRIC: electric,
//   });
//
//   const thread = await injectedCreateThread();
//   const card = await injectedCreateCard({ thread_id: thread.id });
//   const { xml } = await injectedGetCardYDoc(card.id);
//
//   xml.insert(0, [new Y.XmlElement("div")]);
//
//   await new Promise((resolve) => setTimeout(resolve, 1000));
//   const materialized = await electric.db.cards.findUnique({
//     where: { id: card.id },
//   });
//
//   expect(materialized.content).toBe(xml.toString());
// });
//
// testElectric("checkpoint", async ({ electric }) => {
//   const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
//   const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
//   const injectedGetCardYDoc = CardYDoc.init.inject({
//     ELECTRIC: electric,
//   });
//
//   const thread = await injectedCreateThread();
//   const card = await injectedCreateCard({ thread_id: thread.id });
//
//   {
//     const { xml, checkpoint } = await injectedGetCardYDoc(card.id);
//     xml.insert(0, [new Y.XmlElement("div")]);
//     xml.insert(0, [new Y.XmlElement("div")]);
//     await checkpoint(card.id);
//     xml.insert(0, [new Y.XmlElement("div")]);
//     xml.insert(0, [new Y.XmlElement("div")]);
//     await checkpoint(card.id);
//   }
//
//   {
//     const updates = await electric.db.card_ydoc_updates.findMany();
//     expect(updates.length).toBe(2);
//   }
//
//   {
//     const checkpoints = await electric.db.card_ydoc_updates.findMany({
//       where: { checkpoint: true },
//       orderBy: { created_at: "desc" },
//     });
//     expect(checkpoints.length).toBe(2);
//   }
//
//   // merge card updates
//   {
//     await YDocMatelializer.mergeCardUpdates(card.id, 1000, electric);
//     const updates = await electric.db.card_ydoc_updates.findMany();
//     expect(updates.length).toBe(2);
//
//     const { xml } = await injectedGetCardYDoc(card.id);
//
//     expect(xml.toString()).toBe("<div></div><div></div><div></div><div></div>");
//   }
// });
