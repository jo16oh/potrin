import { sql } from "$lib/Utils/utils";

export const getPartialTree = sql`
WITH RECURSIVE thread_tree AS MATERIALIZED (
	SELECT id, title, parent_id, fractional_index, 0 AS depth
	FROM threads
	WHERE id = ? AND deleted = false
	UNION ALL
	SELECT child.id, child.title, child.parent_id, child.fractional_index, parent.depth + 1
	FROM thread_tree AS parent
	JOIN threads AS child ON parent.id = child.parent_id
	WHERE parent.depth < 2 AND deleted = false
	ORDER BY depth DESC, fractional_index ASC, id ASC
),
thread_cards AS MATERIALIZED (
  SELECT cards.id, cards.content, cards.thread_id, cards.fractional_index
  FROM cards
	JOIN thread_tree ON cards.thread_id = thread_tree.id
  WHERE cards.thread_id = thread_tree.id AND deleted = false AND thread_tree.depth < 2
	ORDER BY cards.fractional_index ASC, cards.id ASC
),
breadcrumbs AS (
	SELECT id, title, parent_id, 0 AS breadcrumb
	FROM threads
	WHERE threads.id = (SELECT parent_id FROM thread_tree WHERE depth = 0)
	UNION ALL
	SELECT parent.id, parent.title, parent.parent_id, child.breadcrumb + 1
	FROM threads AS parent
	JOIN breadcrumbs AS child ON child.parent_id = parent.id
),
d2 AS (
	SELECT 
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index
			) AS json,
		fractional_index,
		parent_id
	FROM thread_tree
	WHERE depth = 2
	ORDER BY fractional_index ASC, id ASC
),
d1 AS (
	SELECT 
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'child_threads', (
				SELECT json_group_array(json(json))
				FROM d2
				WHERE d2.parent_id = t1.id
			),
			'cards', (
				SELECT json_group_array(json_object(
					'id', id,
					'content', content,
					'fractional_index', fractional_index
				) ORDER BY fractional_index ASC, id ASC)
				FROM thread_cards
				WHERE thread_cards.thread_id = t1.id
			)
		) AS json,
		fractional_index,
		parent_id
	FROM thread_tree AS t1
	WHERE depth = 1
	ORDER BY fractional_index ASC, id ASC
),
root AS (
	SELECT
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'child_threads', (
				SELECT json_group_array(json(json))
				FROM d1
				WHERE d1.parent_id = t0.id
			),
			'cards', (
				SELECT json_group_array(json_object(
					'id', id,
					'content', content,
					'fractional_index', fractional_index
				) ORDER BY fractional_index ASC, id ASC)
				FROM thread_cards
				WHERE thread_cards.thread_id = t0.id
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
	FROM thread_tree AS t0
	WHERE depth = 0
	ORDER BY fractional_index ASC, id ASC
)
SELECT json FROM root;
`;
