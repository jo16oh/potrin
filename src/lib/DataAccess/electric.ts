import { electrify } from "electric-sql/tauri";
import { ElectricClient } from "electric-sql/client/model";
import { schema } from "../../generated/client";
import type { ElectricConfig, ElectrifyOptions } from "electric-sql";
import Database from "@tauri-apps/plugin-sql";
import { sql } from "$lib/Utils/utils";

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
  ELECTRIC = await wrappedElectrify(electrify, db, schema, config);
}

// wrap electrify function to mock electric client
export const wrappedElectrify = async <T>(
  electrify: ElectrifyFunction<T>,
  db: T,
  schema: Schema,
  config: ElectricConfig,
): Promise<ElectricClient<typeof schema>> => {
  const electric = (await electrify(
    db,
    schema,
    config,
  )) as ElectricClient<Schema>;

  electric.notifier.subscribeToDataChanges((notification) => {
    if (notification.origin === "local") return;
    notification.changes.forEach(async (change) => {
      if (!Array.isArray(change.recordChanges)) return;

      if (change.qualifiedTablename.tablename === "threads") {
        const insertChanges = change.recordChanges
          .filter((c) => c.type === "INSERT")
          .map((c) => c.primaryKey["id"]) as string[];

        await electric.adapter.run({
          sql: sql`
        		DELETE FROM threads as t1
        		WHERE
        			t1.id IN (${insertChanges.map(() => "?").join(", ")})
        			AND t1.parent_id IS NOT NULL
        			AND NOT EXISTS (
								SELECT 1 FROM threads as t2 WHERE t2.id = t1.parent_id
							);
        	`,
          args: insertChanges,
        });

        await electric.adapter.run({
          sql: sql`
          	DELETE FROM cards
          	WHERE
          		thread_id IN (${insertChanges.map(() => "?").join(", ")})
          		AND NOT EXISTS (
								SELECT 1 FROM threads WHERE threads.id = cards.thread_id
							);
          `,
          args: insertChanges,
        });
      }

      if (change.qualifiedTablename.tablename === "cards") {
        const insertChanges = change.recordChanges
          .filter((c) => c.type === "INSERT")
          .map((c) => c.primaryKey["id"]) as string[];

        await electric.adapter.run({
          sql: sql`
          	DELETE FROM cards
          	WHERE
          		id IN (${insertChanges.map(() => "?").join(", ")})
          		AND NOT EXISTS (
								SELECT 1 FROM threads WHERE threads.id = cards.thread_id
							);
          `,
          args: insertChanges,
        });
      }
    });
  });

  return electric;
};
