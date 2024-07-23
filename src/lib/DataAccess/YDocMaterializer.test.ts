import { expect, test } from "vitest";
import { testElectric } from "./testElectric";
import * as Y from "yjs";
import { Card } from "$lib/Models/Card";
import { Thread } from "$lib/Models/Thread";
import { uuidv7 } from "uuidv7";
import Database from "better-sqlite3";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";
import { sql } from "$lib/Utils/utils";

testElectric("YDocMaterializer", async ({ electric }) => {
  const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
  const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
  const thread = await injectedCreateThread();
  const card = await injectedCreateCard({ thread_id: thread.id });

  const ydoc = new Y.Doc();

  ydoc.on("updateV2", (update) => {
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

  const xml = ydoc.getXmlFragment("prosemirror");
  const element = new Y.XmlElement("div");
  element.insert(0, [new Y.XmlText("content")]);
  xml.insert(0, [element]);

  const updates = await electric.db.card_ydoc_updates.findMany();
  expect(updates.length).toBeTruthy();

  await new Promise((resolve) => setTimeout(resolve, 100));
  const updatedCard = await electric.db.cards.findUnique({
    where: { id: card.id },
  });
  expect(updatedCard.content).toBe("<div>content</div>");
});

testElectric("mergeUpdates", async ({ electric }) => {
  const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
  const injectedCreateCard = Card.create.inject({ ELECTRIC: electric });
  const thread = await injectedCreateThread();
  const card = await injectedCreateCard({ thread_id: thread.id });

  const ydoc = new Y.Doc();

  ydoc.on("updateV2", (update) => {
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

  const xml = ydoc.getXmlFragment("prosemirror");

  for (let i = 0; i < 1000; i++) {
    const element = new Y.XmlElement("div");
    element.insert(0, [new Y.XmlText("content")]);
    xml.insert(0, [element]);
  }

  // wait until mergeCardUpdates run
  await new Promise((resolve) => setTimeout(resolve, 100));
  const updates = await electric.db.card_ydoc_updates.findMany();
  expect(updates.length).toBe(901);

  const updatedCard = await electric.db.cards.findUnique({
    where: { id: card.id },
  });
  expect(updatedCard.content).toBe(xml.toString());
});

test("test", async () => {
  const sqlite = new Database(":memory:");
  const electric = await electrify(sqlite, schema);

  electric.notifier.subscribeToDataChanges((notification) => {
    console.log(notification.changes);
  });

  const stmt = sqlite.prepare(
    sql`INSERT INTO users (id, name, created_at) VALUES (?, ?, ?);`,
  );
  stmt.run([uuidv7(), "name", new Date().toString()]);

  await electric.db.users.create({
    data: {
      id: uuidv7(),
      name: "name",
      created_at: new Date(),
    },
  });
  await new Promise((resolve) => setTimeout(resolve, 25));
  await electric.db.users.create({
    data: {
      id: uuidv7(),
      name: "name",
      created_at: new Date(),
    },
  });

  await new Promise((resolve) => setTimeout(resolve, 1000));

  expect(true).toBeTruthy();
});
