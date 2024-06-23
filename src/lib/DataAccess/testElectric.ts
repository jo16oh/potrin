import { test, onTestFinished } from "vitest";
import Database from "better-sqlite3";
import type { Database as SQLite } from "better-sqlite3";
import type { ElectricClient } from "electric-sql/client/model";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";
import { wrappedElectrify } from "$lib/DataAccess/electric";

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
    const sqlite = new Database(":memory:");
    const electric = await createElectric(sqlite);
    await use(electric);
    onTestFinished(async () => {
      await electric.close();
    });
  },
});

export interface TestElectricSync {
  e1: ElectricClient<typeof schema>;
  e2: ElectricClient<typeof schema>;
  token: string;
}

export const testElectricSync = test.extend<TestElectricSync>({
  e1: async ({}, use) => {
    const sqlite = new Database(":memory:");
    const e1 = await createElectric(sqlite);
    await use(e1);
  },
  e2: async ({}, use) => {
    const sqlite = new Database("");
    const e2 = await createElectric(sqlite);
    await use(e2);
  },
  token: token,
});

async function createElectric(sqlite: SQLite) {
  sqlite.pragma("journal_mode = WAL");

  const electric = await wrappedElectrify(electrify, sqlite, schema, config);

  onTestFinished(async () => {
    await electric.close();
  });

  return electric;
}
