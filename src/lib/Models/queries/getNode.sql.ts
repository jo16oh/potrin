import { sql } from "$lib/Utils/utils";

export const getNode = sql`
SELECT json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'cards', (
				SELECT json_group_array(json_object(
					'id', id,
					'content', content,
					'fractional_index', fractional_index
				) ORDER BY fractional_index ASC, id ASC)
				FROM cards
				WHERE cards.thread_id = threads.id
			)
		) AS json
	FROM threads
	WHERE id = ?;
`;
