import { sql } from "$lib/Utils/utils";

export const getNode = sql`
WITH RECURSIVE breadcrumbs AS (
	SELECT id, title, parent_id, 0 AS breadcrumb
	FROM threads
	WHERE threads.id = ?
	UNION ALL
	SELECT parent.id, parent.title, parent.parent_id, child.breadcrumb + 1
	FROM threads AS parent
	JOIN breadcrumbs AS child ON child.parent_id = parent.id
)
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
			),
			'breadcrumbs', (
				SELECT json_group_array(json_object(
						'id', id,
						'title', title,
						'parent_id', parent_id
					) ORDER BY breadcrumb DESC)
					FROM breadcrumbs
				)
		) AS json
	FROM threads
	WHERE id = (SELECT id FROM breadcrumbs WHERE breadcrumb = 0);
`;
