import type { ElectricClient } from "electric-sql/client/model";
import type { schema } from "../../generated/client";
import { sql } from "$lib/Utils/utils";
import { murmurHash } from "ohash";
import * as Y from "yjs";

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

    const content = ydoc.getXmlFragment().toString();

    await electric.db.cards.update({
      where: { id: card.id },
      data: {
        content: content,
        last_materialized_hash: hash,
        updated_at: new Date(),
      },
    });
  },
};
