CREATE TABLE outline_links(
    id_from BLOB REFERENCES outline(id),
    id_to BLOB REFERENCES outline(id),
    PRIMARY KEY (id_from, id_to)
) STRICT;

CREATE INDEX outline_links$from_idx ON outline_links(id_from);
CREATE INDEX outline_links$to_idx ON outline_links(id_to);

CREATE TABLE card_links (
    id_from BLOB REFERENCES card(id),
    id_to BLOB REFERENCES outline(id),
    PRIMARY KEY (id_from, id_to)
) STRICT;

CREATE INDEX card_links$from_idx ON card_links(id_from);
CREATE INDEX card_links$to_idx ON card_links(id_to);

CREATE TABLE card_quotes (
    id_from BLOB REFERENCES card(id),
    id_to BLOB REFERENCES card(id),
    PRIMARY KEY (id_from, id_to)
) STRICT;

CREATE INDEX card_quotes$from_idx ON card_quotes(id_from);
CREATE INDEX card_quotes$to_idx ON card_quotes(id_to);
