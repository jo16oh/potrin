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

CREATE TABLE quotes (
    card_id BLOB REFERENCES cards(id) ON DELETE CASCADE PRIMARY KEY,
    quote BLOB REFERENCES cards(id) ON DELETE SET NULL,
    version_id BLOB REFERENCES versions(id) ON DELETE RESTRICT
) STRICT;

CREATE INDEX quotes$quote_idx ON quotes(quote);
CREATE INDEX quotes$version_id_idx ON quotes(version_id);
