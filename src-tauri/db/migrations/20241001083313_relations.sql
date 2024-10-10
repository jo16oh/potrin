CREATE TABLE outline_links(
    id_from BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    id_to BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    PRIMARY KEY (id_from, id_to)
) STRICT;

CREATE INDEX outline_links$from_idx ON outline_links(id_from);
CREATE INDEX outline_links$to_idx ON outline_links(id_to);

CREATE TABLE card_links (
    id_from BLOB REFERENCES cards(id) ON DELETE CASCADE,
    id_to BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    PRIMARY KEY (id_from, id_to)
) STRICT;

CREATE INDEX card_links$from_idx ON card_links(id_from);
CREATE INDEX card_links$to_idx ON card_links(id_to);
