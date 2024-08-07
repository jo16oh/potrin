import { describe, expect } from "vitest";
import { Thread } from "$lib/Models/Thread";
import { testElectric, testElectricSync } from "$lib/DataAccess/testElectric";
import { uuidv7 } from "uuidv7";

describe("Thread", () => {
  testElectric("create thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    await injectedCreateThread();
    expect((await electric.db.threads.findMany()).length).toBe(1);
  });

  testElectric("update thread", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedUpdateThread = Thread.update.inject({ ELECTRIC: electric });
    const thread = await injectedCreateThread();
    await injectedUpdateThread({ ...thread, title: "updated" });
    expect(
      (await electric.db.threads.findUnique({ where: { id: thread.id } }))?.[
        "title"
      ],
    ).toBe("updated");
  });

  testElectric("thread title must be unique", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedUpdateThread = Thread.update.inject({ ELECTRIC: electric });

    await injectedCreateThread({ title: "title" });
    expect(injectedCreateThread({ title: "title" })).rejects.toThrow();

    const thread = await injectedCreateThread({ title: "title2" });
    expect(
      injectedUpdateThread({ id: thread.id, title: "title" }),
    ).rejects.toThrow();

    expect(
      injectedUpdateThread({ id: thread.id, title: "title2" }),
    ).resolves.toBeTruthy();
  });

  testElectricSync(
    "keep thread title unique when synced using trigger",
    async ({ e1, e2, token }) => {
      // console.log(e1);
      const createThread1 = Thread.create.inject({ ELECTRIC: e1 });
      const createThread2 = Thread.create.inject({ ELECTRIC: e2 });

      await createThread1({ title: "title" });
      await createThread2({ title: "title" });

      await e1.connect(token);
      const s1 = await e1.db.threads.sync();
      await s1.synced;
      await e2.connect(token);
      const s2 = await e2.db.threads.sync();
      await s2.synced;
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const res = await e1.db.threads.findMany({ where: { title: "title" } });
      expect(res.length).toBe(1);
    },
    100000,
  );

  testElectric("check parent existence", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const res = injectedCreateThread({
      parent_id: uuidv7(),
    });

    expect(res).rejects.toThrow();
  });

  testElectric("deletePhysical should cascade down", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedDeleteThread = Thread.deletePhysical.inject({
      ELECTRIC: electric,
    });

    const thread = await injectedCreateThread();
    const child = await injectedCreateThread({ parent_id: thread.id });
    await injectedCreateThread({ parent_id: child.id });

    const res = await electric.db.threads.findMany();
    expect(res.length).toBe(3);

    await injectedDeleteThread(thread.id);
    await new Promise((resolve) => setTimeout(resolve, 0));

    const res2 = await electric.db.threads.findMany();
    expect(res2.length).toBe(0);
  });

  testElectric("delete logical", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedDeleteThreadLogical = Thread.deleteLogical.inject({
      ELECTRIC: electric,
    });
    const thread = await injectedCreateThread();
    await injectedDeleteThreadLogical(thread.id);
    const deleted = await electric.db.threads.findUnique({
      where: { id: thread.id },
    });
    expect(deleted.deleted).toBeTruthy();
  });
});
