WITH RECURSIVE tree AS (
	SELECT id, parent_id, fractional_index, 0 AS depth
	FROM outlines
	WHERE id = ? AND is_deleted = 0
	UNION
	SELECT child.id, child.parent_id, child.fractional_index, parent.depth + 1 AS depth
	FROM outlines AS child
	INNER JOIN tree AS parent ON child.parent_id = parent.id
	WHERE child.is_deleted = 0
	UNION
	SELECT parents_siblings.id, parents_siblings.parent_id, parents_siblings.fractional_index, child.depth - 1 AS depth
	FROM outlines AS parent
	INNER JOIN tree AS child ON child.parent_id = parent.id
  INNER JOIN outlines AS parents_siblings ON 
    parents_siblings.id = (
      SELECT id 
      FROM outlines
      WHERE 
      parent.parent_id = parents_siblings.parent_id AND
      parent.fractional_index <= parents_siblings.fractional_index AND
      parent.id < parents_siblings.id AND
      parent.id != ? AND
      parents_siblings.is_deleted = 0
      LIMIT 1
    )
	WHERE parents_siblings.is_deleted = 0
	ORDER BY depth DESC, fractional_index ASC, id ASC
	LIMIT ?
)
SELECT id FROM tree WHERE id != ?;

-- args
-- 1: root_id OR previous_last_id
-- 2: root_id
-- 3: limit 
-- 4: if 1 is root_id, then NULL, else previous_last_id
