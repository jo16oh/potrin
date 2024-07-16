import { test, onTestFinished } from "vitest";
import Database from "better-sqlite3";
import type { Database as SQLite } from "better-sqlite3";
import type { ElectricClient } from "electric-sql/client/model";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";
import { createElectric } from "$lib/DataAccess/electric";
import { sql } from "$lib/Utils/utils";
import * as fs from "node:fs";

const token =
  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

const config = {
  url: "http://localhost:5134",
  debug: false,
};

export interface TestElectric {
  electric: ElectricClient<typeof schema>;
}

export const testElectric = test.extend<TestElectric>({
  electric: async ({}, use) => {
    const sqlite = initSQLite(":memory:");
    const electric = await createElectric(electrify, sqlite, schema, config);
    onTestFinished(async () => {
      await electric.close();
      sqlite.close();
    });
    await use(electric);
  },
});

export interface TestElectricSync {
  e1: ElectricClient<typeof schema>;
  e2: ElectricClient<typeof schema>;
  token: string;
}

export const testElectricSync = test.extend<TestElectricSync>({
  e1: async ({}, use) => {
    const path = "e1.db";
    removeFile(path);
    const sqlite = initSQLite(path);
    const e1 = await createElectric(electrify, sqlite, schema, config);
    await cleanup(e1, sqlite, path);
    await use(e1);
  },

  e2: async ({}, use) => {
    const path = "e2.db";
    removeFile(path);
    const sqlite = initSQLite(path);
    const e2 = await createElectric(electrify, sqlite, schema, config);
    await cleanup(e2, sqlite, path);
    await use(e2);
  },

  token: token,
});

function initSQLite(path: string): SQLite {
  const sqlite = new Database(path);
  sqlite.pragma("journal_mode = WAL");
  return sqlite;
}

async function cleanup(
  e: ElectricClient<typeof schema>,
  sqlite: SQLite,
  path: string,
) {
  await e.connect(token);
  const cards = await e.db["cards"].sync();
  const threads = await e.db["threads"].sync();
  await threads.synced;
  await cards.synced;
  await e.db["users"].deleteMany();
  await e.db["pots"].deleteMany();
  await e.db["threads"].deleteMany();
  await e.db["thread_checkpoints"].deleteMany();
  await e.db["cards"].deleteMany();
  await e.db["card_ydoc_updates"].deleteMany();
  await e.db["card_checkpoints"].deleteMany();
  await e.db["thread_card_checkpoints"].deleteMany();
  await e.adapter.run({ sql: sql`DELETE FROM changed_threads;` });
  await e.adapter.run({ sql: sql`DELETE FROM changed_cards;` });

  // wait until all delete operations sent to the electric-sync-service
  await new Promise((resolve) => setTimeout(resolve, 1000));

  onTestFinished(async () => {
    await cleanup(e, sqlite, path);
    await e.close();
    sqlite.close();
    removeFile(path);
  });
}

function removeFile(path: string) {
  fs.access(path, fs.constants.F_OK, (err) => {
    if (!err) {
      fs.unlinkSync(path);
    }
  });
}
