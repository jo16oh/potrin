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

  await createLogTableAndTrigger(electric);
  await reconcileThreads(electric);
  await reconcileCards(electric);

  electric.notifier.subscribeToDataChanges((notification) => {
    // @ts-expect-error to avoid error in test
    if (typeof process !== "undefined" && !electric.adapter.db.open) return;

    notification.changes.forEach(async (change) => {
      if (!Array.isArray(change.recordChanges)) return;

      if (change.qualifiedTablename.tablename === "threads") {
        await reconcileThreads(electric);
      }

      if (change.qualifiedTablename.tablename === "cards") {
        await reconcileCards(electric);
      }
    });
  });

  return electric;
};

async function createLogTableAndTrigger(
  electric: ElectricClient<typeof schema>,
) {
  await electric.adapter.runInTransaction(
    {
      sql: sql`
			CREATE TABLE IF NOT EXISTS changed_threads (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				thread_id TEXT,
				deleted BOOLEAN
			);
		`,
    },
    {
      sql: sql`
			CREATE TABLE IF NOT EXISTS changed_cards (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				card_id TEXT,
				deleted BOOLEAN
			);
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_thread_change_after_insert
			AFTER INSERT ON threads
			BEGIN
				INSERT
				INTO changed_threads (
					thread_id,
					deleted
				)
				VALUES (
					NEW.id,
					0
				);
			END;
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_thread_change_after_update
			AFTER UPDATE ON threads
			BEGIN
				INSERT
				INTO changed_threads (
					thread_id,
					deleted
				)
				VALUES (
					NEW.id,
					0
				);
			END;
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_thread_change_after_delete
			AFTER DELETE ON threads
			BEGIN
				INSERT
				INTO changed_threads (
					thread_id,
					deleted
				)
				VALUES (
					OLD.id,
					1
				);
			END;
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_card_change_after_insert
			AFTER INSERT ON cards
			BEGIN
				INSERT
				INTO changed_cards (
					card_id,
					deleted
				)
				VALUES (
					NEW.id,
					0
				);
			END;
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_card_change_after_update
			AFTER UPDATE ON cards
			BEGIN
				INSERT
				INTO changed_cards (
					card_id,
					deleted
				)
				VALUES (
					NEW.id,
					0
				);
			END;
		`,
    },
    {
      sql: sql`
			CREATE TRIGGER IF NOT EXISTS log_card_change_after_delete
			AFTER DELETE ON cards
			BEGIN
				INSERT
				INTO changed_cards (
					card_id,
					deleted
				)
				VALUES (
					OLD.id,
					1
				);
			END;
		`,
    },
  );
}

async function reconcileThreads(electric: ElectricClient<typeof schema>) {
  const changed_threads = await electric.db.rawQuery({
    sql: sql`SELECT id, thread_id, deleted FROM changed_threads;`,
  });
  const deleted_ids = new Set(
    changed_threads
      .filter((i) => i["deleted"])
      .map((i) => i["thread_id"] as string),
  );
  const changed_ids = new Set(
    changed_threads
      .filter((i) => !i["deleted"])
      .map((i) => i["thread_id"] as string)
      .filter((i) => !deleted_ids.has(i)),
  );

  // delete orphans
  await electric.adapter.runInTransaction(
    {
      sql: sql`
				CREATE TEMPORARY TABLE IF NOT EXISTS tmp_thread_tree (id TEXT);
			`,
    },
    {
      sql: sql`
				WITH RECURSIVE thread_tree AS (
					SELECT threads.id, threads.parent_id 
					FROM threads
					WHERE id IN (
						SELECT id
						FROM threads 
						WHERE 
						id IN (${Array.from(changed_ids)
              .map(() => "?")
              .join(", ")})
						AND parent_id IS NOT NULL
						AND NOT EXISTS (
							SELECT 1
							FROM threads AS t2
							WHERE t2.id = threads.parent_id
						)
					)
					UNION ALL
					SELECT child.id, child.parent_id 
					FROM thread_tree AS parent
					JOIN threads AS child ON parent.id = child.parent_id
				)
				INSERT INTO tmp_thread_tree (id)
					SELECT id FROM thread_tree;
			`,
      args: [...Array.from(changed_ids)],
    },
    {
      sql: sql`
				DELETE 
				FROM threads
				WHERE id IN (SELECT id FROM tmp_thread_tree);
			`,
    },
    {
      sql: sql`
				DELETE 
				FROM cards
				WHERE thread_id IN (SELECT id FROM tmp_thread_tree);
			`,
    },
    {
      sql: sql`DROP TABLE tmp_thread_tree;`,
    },
  );

  // update duplicated title
  await electric.adapter.runInTransaction({
    sql: sql`
			UPDATE threads
			SET 
			title = title || '-' || id
			WHERE 
			id IN (${Array.from(changed_ids)
        .map(() => "?")
        .join(", ")})							
			AND EXISTS (
				SELECT 1 
				FROM threads as t2
				WHERE 
				t2.id <> threads.id
				AND (t2.parent_id IS NULL OR t2.parent_id = threads.parent_id)
				AND t2.title = threads.title
			)
			AND (title, created_at) NOT IN (
				SELECT title, MIN(created_at)
				FROM threads
				GROUP BY title
			);
		`,
    args: [...Array.from(changed_ids)],
  });

  // TODO: update search index

  // cleanup used rows from changed_threads table
  await electric.adapter.run({
    sql: sql`
			DELETE 
			FROM changed_threads 
			WHERE id
			IN (
			${changed_threads.map((i) => i["id"])}
			);
		`,
  });
}

async function reconcileCards(electric: ElectricClient<typeof schema>) {
  const changed_cards = await electric.db.rawQuery({
    sql: sql`SELECT id, card_id, deleted FROM changed_cards;`,
  });
  const deleted_ids = new Set(
    changed_cards
      .filter((i) => i["deleted"])
      .map((i) => i["card_id"] as string),
  );
  const changed_ids = new Set(
    changed_cards
      .filter((i) => !i["deleted"])
      .map((i) => i["card_id"] as string)
      .filter((i) => !deleted_ids.has(i)),
  );

  // delete orphans
  await electric.adapter.run({
    sql: sql`
			DELETE 
			FROM cards 
			WHERE 
				id IN (${Array.from(changed_ids)
          .map(() => "?")
          .join(", ")})
				AND NOT EXISTS (
					SELECT 1
					FROM threads
					WHERE threads.id = cards.thread_id
				);
		`,
    args: Array.from(changed_ids),
  });

  // TODO: update search index

  // cleanup used rows from changed_threads table
  await electric.adapter.run({
    sql: sql`
			DELETE 
			FROM changed_cards 
			WHERE id
			IN (
				${changed_cards.map((i) => i["id"])}
			);
		`,
  });
}
