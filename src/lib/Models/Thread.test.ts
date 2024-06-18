import { describe, expect } from "vitest";
import { Thread } from "$lib/Models/Thread";
import { testWithElectric } from "$lib/TestUtils/testWithElectric";

describe("Thread", async () => {
  testWithElectric("create thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    await injectedCreateThread();
    expect((await electric.db.threads.findMany()).length).toBe(1);
  });
  testWithElectric("update thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedUpdateThread = Thread.update.inject({ ELECTRIC: electric });
    const thread = await injectedCreateThread();
    await injectedUpdateThread({ ...thread, title: "updated" });
    expect(
      (await electric.db.threads.findUnique({ where: { id: thread.id } }))
        .title,
    ).toBe("updated");
  });
});
