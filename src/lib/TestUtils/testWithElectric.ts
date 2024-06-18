import { test, onTestFinished } from "vitest";
import Database from "better-sqlite3";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";
import type { ElectricClient } from "electric-sql/client/model";

const config = {
  url: "http://localhost:5134",
  debug: false,
};

export interface TestWithElectric {
  electric: ElectricClient<typeof schema>;
}

export const testWithElectric = test.extend<TestWithElectric>({
  electric: async ({}, use) => {
    const sqlite = new Database(":memory:");
    sqlite.pragma("journal_mode = WAL");
    const electric = await electrify(sqlite, schema, config);
    await use(electric);
    onTestFinished(async () => {
      await electric.close();
    });
  },
});
