import { electrify } from "electric-sql/tauri";
import { ElectricClient } from "electric-sql/client/model";
import { schema } from "../../generated/client";
import Database from "@tauri-apps/plugin-sql";
import { ok } from "neverthrow";
import { execAsyncThrowable } from "$lib/Utils/neverthrow-utils";

const config = {
  url: "http://localhost:5133",
  debug: false,
};

export let ELECTRIC: undefined | ElectricClient<typeof schema>;

export function init() {
  return execAsyncThrowable(() => Database.load("sqlite:electric.db"))
    .andThen((db) => ok(Object.assign(db, { name: "electric.db" })))
    .andThen((db) => execAsyncThrowable(() => electrify(db, schema, config)))
    .map((e) => {
      ELECTRIC = e;
    });
}
