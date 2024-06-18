import { electrify } from "electric-sql/tauri";
import { ElectricClient } from "electric-sql/client/model";
import { schema } from "../../generated/client";
import Database from "@tauri-apps/plugin-sql";

const config = {
  url: "http://localhost:5133",
  debug: false,
};

export let ELECTRIC: undefined | ElectricClient<typeof schema>;

export async function init() {
  const sqlite = await Database.load("sqlite:electric.db");
  const db = Object.assign(sqlite, { name: "electric.db" });
  ELECTRIC = await electrify(db, schema, config);
}
