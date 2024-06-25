import { describe, expect } from "vitest";
import { Thread } from "$lib/Models/Thread";
import { testElectric, testElectricSync } from "$lib/DataAccess/testElectric";
import { Card } from "./Card";

describe("Thread", async () => {
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

  testElectric("check parent existence", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const res = injectedCreateThread({
      parent_id: crypto.randomUUID(),
    });

    expect(res).rejects.toThrow();
  });

  testElectric("cascade delete", async ({ electric }) => {
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
    // await electric.db.threads.delete({ where: { id: thread.id } });
    await new Promise((resolve) => setTimeout(resolve, 1000));

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

  testElectric("delete physical", async ({ electric }) => {
    const injectedCreateThread = Thread.create.inject({ ELECTRIC: electric });
    const injectedDeleteThreadPhysical = Thread.deletePhysical.inject({
      ELECTRIC: electric,
    });
    const thread = await injectedCreateThread();
    await injectedDeleteThreadPhysical(thread.id);
    const deleted = await electric.db.threads.findUnique({
      where: { id: thread.id },
    });
    expect(deleted).toBeFalsy();
  });
});
