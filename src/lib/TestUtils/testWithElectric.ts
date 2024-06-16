import { test } from "vitest";
import Database from "better-sqlite3";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";
import type { ElectricClient } from "electric-sql/client/model";

const config = {
  url: "http://localhost:5134",
  debug: false,
};

const sqlite = new Database("");
sqlite.pragma("journal_mode = WAL");

const electric = await electrify(sqlite, schema, config);

export interface TestWithElectric {
  electric: ElectricClient<typeof schema>;
}

export const testWithElectric = test.extend<TestWithElectric>({
  electric: async ({}, use) => {
    await electric.db.cards.deleteMany();
    await electric.db.threads.deleteMany();
    await use(electric);
  },
});
