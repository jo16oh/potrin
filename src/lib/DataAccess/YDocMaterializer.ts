import type { ElectricClient } from "electric-sql/client/model";
import type { schema } from "../../generated/client";
import { sql } from "$lib/Utils/utils";
import { murmurHash } from "ohash";
import * as Y from "yjs";
import { uuidv7 } from "uuidv7";

export const YDocMatelializer = {
  async init(electric: ElectricClient<typeof schema>) {
    electric.notifier.subscribeToDataChanges((notification) => {
      // @ts-expect-error to avoid error in test
      if (typeof process !== "undefined" && !electric.adapter.db.open) return;
      if (notification.origin !== "local") return;

      notification.changes.forEach(async (change) => {
        if (change.qualifiedTablename.tablename !== "card_ydoc_updates") return;
        if (!change.recordChanges) return;

        const ydocUpdateIds = change.recordChanges.map(
          (i) => i.primaryKey["id"],
        ) as string[];

        const cardIds = (
          await electric.db.rawQuery({
            sql: sql`
							SELECT DISTINCT card_id 
							FROM card_ydoc_updates 
							WHERE id IN (${ydocUpdateIds.map(() => "?").join(", ")});
						`,
            args: ydocUpdateIds,
          })
        ).map((update) => update["card_id"]);

        const cards = (await electric.db.cards.findMany({
          select: { id: true, last_materialized_hash: true },
          where: { id: { in: cardIds } },
        })) as Array<{ id: string; last_materialized_hash: string }>;

        cards.forEach((c) => {
          this.materializeCard(c, electric);
        });
      });
    });
  },

  async materializeCard(
    card: { id: string; last_materialized_hash: string },
    electric: ElectricClient<typeof schema>,
  ) {
    const ydocUpdates = await electric.db.card_ydoc_updates.findMany({
      where: { card_id: card.id },
    });

    const hash = murmurHash(
      ydocUpdates
        .map((i) => i["id"])
        .sort()
        .join(":"),
    ).toString();

    if (card.last_materialized_hash === hash) return;

    const ydoc = new Y.Doc();

    ydocUpdates.forEach((update) => {
      Y.applyUpdateV2(ydoc, update["data"]);
    });

    const content = ydoc.getXmlFragment("prosemirror").toString();

    await electric.db.cards.update({
      where: { id: card.id },
      data: {
        content: content,
        last_materialized_hash: hash,
        updated_at: new Date(),
      },
    });
  },

  async mergeCardUpdates(
    cardId: string,
    mergeTargetLength: number,
    electric: ElectricClient<typeof schema>,
  ) {
    const checkpoints = await electric.db.card_ydoc_updates.findMany({
      where: { card_id: cardId, checkpoint: true },
      orderBy: { created_at: "asc" },
    });

    let mergedUpdatesLengh: number = 0;

    for (const c of checkpoints) {
      if (mergedUpdatesLengh >= mergeTargetLength) return;

      const mergeTargets = await electric.db.card_ydoc_updates.findMany({
        where: {
          card_id: cardId,
          checkpoint: false,
          created_at: { lte: c["created_at"] },
        },
        take: mergeTargetLength - mergedUpdatesLengh,
        orderBy: { created_at: "asc" },
      });
      if (!mergeTargets.length) return;

      const mergedUpdate = Y.mergeUpdatesV2([
        c["data"],
        ...mergeTargets.map((i) => i["data"]),
      ]);

      await electric.adapter.runInTransaction(
        {
          sql: sql`
						DELETE 
						FROM card_ydoc_updates 
						WHERE id IN (${mergeTargets.map(() => "?").join(", ")});
					`,
          args: mergeTargets.map((c) => c["id"]),
        },
        {
          sql: sql`
						UPDATE card_ydoc_updates
						SET data = ?
						WHERE id = ?;
					`,
          args: [mergedUpdate, c["id"]],
        },
      );

      mergedUpdatesLengh = mergedUpdatesLengh + mergeTargets.length;
    }

    if (mergedUpdatesLengh >= mergeTargetLength) return;

    const mergeTargets = await electric.db.card_ydoc_updates.findMany({
      where: {
        card_id: cardId,
        checkpoint: false,
      },
      take: mergeTargetLength - mergedUpdatesLengh,
      orderBy: { created_at: "asc" },
    });
    if (!mergeTargets.length) return;

    const mergedUpdate = Y.mergeUpdatesV2(mergeTargets.map((i) => i["data"]));

    await electric.adapter.runInTransaction(
      {
        sql: sql`
					DELETE 
					FROM card_ydoc_updates 
					WHERE id IN (${mergeTargets.map(() => "?").join(", ")});
				`,
        args: mergeTargets.map((c) => c["id"]),
      },
      {
        sql: sql`
					INSERT
					INTO card_ydoc_updates (id, card_id, data, checkpoint, created_at)
					VALUES (?, ?, ?, ?, ?)
				`,
        args: [uuidv7(), cardId, mergedUpdate, 0, new Date().toString()],
      },
    );
  },
} as const;
