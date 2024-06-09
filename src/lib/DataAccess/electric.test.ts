import { expect, test } from "vitest";
import Database from "better-sqlite3";
import { electrify } from "electric-sql/node";
import { schema } from "../../generated/client";

const config = {
  url: "http://localhost:5134",
  debug: false,
};

const DUMMY_TOKEN =
  "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.P9Klv_0x0vk32T1z4PFqZaKDeF2DFacQJiOxqjXGS48";

const sqlite = new Database(":memory:");
sqlite.pragma("journal_mode = WAL");
const electric = await electrify(sqlite, schema, config);

test("electric test instance initialization", async () => {
  expect(electric.db).toBeTruthy();
});

test("electric connection", async () => {
  await electric.connect(DUMMY_TOKEN);
  expect(electric.isConnected).toBeTruthy();
});
