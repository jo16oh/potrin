import { sql } from "$lib/Utils/utils";

export type ThreadTree = {
  id: string;
  title: string;
  fractional_index: string;
  parent_thread: ThreadTree;
  child_threads?: ThreadTree[];
};

export const query = sql`
WITH RECURSIVE thread_tree AS MATERIALIZED (
	SELECT id, title, parent_thread, fractional_index, 0 AS depth
	FROM threads
	WHERE id = ?
	UNION ALL
	SELECT child.id, child.title, child.parent_thread, child.fractional_index, parent.depth + 1
	FROM thread_tree AS parent
	JOIN threads AS child ON parent.id = child.parent_thread
	WHERE parent.depth < 7
	ORDER BY depth DESC
),
d6 AS (
	SELECT
		json_object(
		'id', id,
		'title', title,
		'fractional_index', fractional_index
		) AS json,
		fractional_index,
		parent_thread
	FROM thread_tree AS t6
	WHERE depth = 6
	ORDER BY fractional_index ASC, id ASC
	),
d5 AS (
	SELECT
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'child_threads', (
				SELECT json_group_array(json(json))
				FROM d6
				WHERE d6.parent_thread = t5.id
			)
		) AS json,
		fractional_index,
		parent_thread
	FROM thread_tree AS t5
	WHERE depth = 5
	ORDER BY fractional_index ASC, id ASC
),
d4 AS (
	SELECT
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'child_threads', (
				SELECT json_group_array(json(json))
				FROM d5
				WHERE d5.parent_thread = t4.id
			)
		) AS json,
		fractional_index,
		parent_thread
	FROM thread_tree AS t4
	WHERE depth = 4
	ORDER BY fractional_index ASC, id ASC
),
d3 AS (
	SELECT 
		json_object(
			'id', id,
			'title', title,
			'fractional_index', fractional_index,
			'child_threads', (
				SELECT json_group_array(json(json))
				FROM d4
				WHERE d4.parent_thread = t3.id
			)
		) AS json,
		fractional_index,
		parent_thread
	FROM thread_tree AS t3
	WHERE depth = 3
	ORDER BY fractional_index ASC, id ASC
),
d2 AS (
SELECT 
	json_object(
		'id', id,
		'title', title,
		'fractional_index', fractional_index,
		'child_threads', (
			SELECT json_group_array(json(json))
			FROM d3
			WHERE d3.parent_thread = t2.id
		)
		) AS json,
		fractional_index,
		parent_thread
		FROM thread_tree AS t2
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
			WHERE d2.parent_thread = t1.id
		)
	) AS json,
	fractional_index,
	parent_thread
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
			WHERE d1.parent_thread = t0.id
		)
	) AS json
	FROM thread_tree AS t0
	WHERE depth = 0
	ORDER BY fractional_index ASC, id ASC
)
SELECT json FROM root;
`;
