import { expect, afterAll, test, describe } from "vitest";
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

export const ELECTRIC_TEST = await electrify(sqlite, schema, config);

describe("electric test instance initialization", async () => {
  test("initialization", async () => {
    expect(ELECTRIC_TEST.db).toBeTruthy();
  });

  test("connection", async () => {
    await ELECTRIC_TEST.connect(DUMMY_TOKEN);
    expect(ELECTRIC_TEST.isConnected).toBeTruthy();
  });
});

afterAll(async () => {
  await ELECTRIC_TEST.db.cards.deleteMany();
  await ELECTRIC_TEST.db.threads.deleteMany();
});
