import { describe, expect } from "vitest";
import { Thread } from "$lib/Models/Thread";
import { uuidv7 } from "uuidv7";
import { testWithElectric } from "$lib/TestUtils/testWithElectric";

describe("Thread", async () => {
  const id = uuidv7();
  testWithElectric("create thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    await injectedCreateThread({ id: id });
    expect((await electric.db.threads.findMany()).length).toBe(1);
  });
  testWithElectric("update thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedUpdateThread = Thread.update.inject({ ELECTRIC: electric });
    await injectedCreateThread({ id: id });
    await injectedUpdateThread(id, { title: "updated" });
    expect(
      (await electric.db.threads.findUnique({ where: { id: id } })).title,
    ).toBe("updated");
  });
});
