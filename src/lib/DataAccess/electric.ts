import { electrify } from "electric-sql/tauri";
import { ElectricClient } from "electric-sql/client/model";
import { schema } from "../../generated/client";
import type { ElectricConfig, ElectrifyOptions } from "electric-sql";
import Database from "@tauri-apps/plugin-sql";
import { TableReconciler } from "./TableReconciler";
import { YDocMatelializer } from "./YDocMaterializer";

type Schema = typeof schema;

type ElectrifyFunction<T> = (
  db: T,
  schema: Schema,
  config: ElectricConfig,
  options?: ElectrifyOptions,
) => Promise<ElectricClient<Schema>>;

const config = {
  url: "http://localhost:5133",
  debug: false,
};

export let ELECTRIC: undefined | ElectricClient<Schema>;

export async function init() {
  const sqlite = await Database.load("sqlite:electric.db");
  const db = Object.assign(sqlite, { name: "electric.db" });
  ELECTRIC = await createElectric(electrify, db, schema, config);
}

// wrap electrify function to mock electric client
export const createElectric = async <T>(
  electrify: ElectrifyFunction<T>,
  db: T,
  schema: Schema,
  config: ElectricConfig,
): Promise<ElectricClient<Schema>> => {
  const electric = await electrify(db, schema, config);
  await TableReconciler.init(electric);
  await YDocMatelializer.init(electric);
  return electric;
};
