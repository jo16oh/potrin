// import * as Y from "yjs";
// import { depend } from "velona";
// import { ELECTRIC } from "$lib/DataAccess/electric";
// import { uuidv7 } from "uuidv7";
// import { sql } from "$lib/Utils/utils";
//
// export type CardYDoc = {
//   id: string;
//   xml: Y.XmlFragment;
//   undoManager: Y.UndoManager;
//   checkpoint: (cardId: string) => Promise<void>;
//   storePendingUpdates: () => Promise<void>;
// };
//
// type options = {
//   storeTimeout: number;
// };
//
// export const CardYDoc = {
//   init: depend(
//     { ELECTRIC },
//     async (
//       { ELECTRIC },
//       cardId: string,
//       options: options = { storeTimeout: 1000 },
//     ): Promise<CardYDoc> => {
//       if (!ELECTRIC) throw new Error("electric has not initialized yet");
//       const ydoc = new Y.Doc();
//       const xml = ydoc.getXmlFragment("prosemirror");
//       const undoManager = new Y.UndoManager(xml);
//
//       const initialUpdates = (
//         await ELECTRIC.db.card_ydoc_updates.findMany({
//           select: { data: true },
//           where: { card_id: cardId },
//         })
//       ).map((c) => c["data"]);
//
//       for (const update of initialUpdates) {
//         Y.applyUpdateV2(ydoc, update);
//       }
//
//       let pendingUpdates: Uint8Array[] = [];
//       let timeoutId: null | ReturnType<typeof setTimeout> = null;
//
//       ydoc.on("updateV2", (update) => {
//         pendingUpdates.push(update);
//         if (timeoutId) return;
//         timeoutId = setTimeout(() => {
//           storePendingUpdates();
//         }, options.storeTimeout);
//       });
//
//       async function storePendingUpdates() {
//         if (!ELECTRIC) throw new Error("electric has not initialized yet");
//         if (timeoutId) {
//           clearTimeout(timeoutId);
//           timeoutId = null;
//         }
//
//         const update = Y.mergeUpdatesV2(pendingUpdates);
//         pendingUpdates = [];
//
//         await ELECTRIC.db.card_ydoc_updates.create({
//           data: {
//             id: uuidv7(),
//             card_id: cardId,
//             data: update,
//             checkpoint: false,
//             created_at: new Date(),
//           },
//         });
//       }
//
//       async function checkpoint(cardId: string) {
//         if (!ELECTRIC) throw new Error("electric has not initialized yet");
//
//         const content = xml.toString();
//         await storePendingUpdates();
//         await ELECTRIC.adapter.runInTransaction(
//           {
//             sql: sql`
// 							CREATE TEMPORARY TABLE IF NOT EXISTS latest_ydoc_update (id)
// 						`,
//           },
//           {
//             sql: sql`
// 							INSERT INTO latest_ydoc_update (id)
// 							SELECT id
// 							FROM card_ydoc_updates
// 							WHERE card_id = ?
// 							ORDER BY created_at DESC
// 							LIMIT 1;
// 						`,
//             args: [cardId],
//           },
//           {
//             sql: sql`
// 							UPDATE card_ydoc_updates
// 							SET checkpoint = 1
// 							WHERE id = (
// 								SELECT id
// 								FROM latest_ydoc_update
// 							);
// 						`,
//           },
//           {
//             sql: sql`
// 							INSERT
// 							INTO card_checkpoints (id, card_id, ydoc_id, fractional_index, content)
// 							VALUES (
// 								?, ?,
// 								(
// 									SELECT id
// 									FROM latest_ydoc_update
// 								),
// 								(
// 									SELECT fractional_index
// 									FROM cards
// 									WHERE id = ?
// 								),
// 								?
// 							);
// 						`,
//             args: [uuidv7(), cardId, cardId, content],
//           },
//           {
//             sql: sql`
// 							DROP TABLE latest_ydoc_update;
// 						`,
//           },
//         );
//       }
//
//       return {
//         id: cardId,
//         xml,
//         undoManager,
//         checkpoint,
//         storePendingUpdates,
//       } as const;
//     },
//   ),
// };
