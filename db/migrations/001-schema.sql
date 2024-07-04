CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
ALTER TABLE users ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS pots (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  owner UUID REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL
);
ALTER TABLE pots ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS threads (
  id UUID PRIMARY KEY,
  author UUID REFERENCES users(id),
  pot_id UUID REFERENCES pots(id) ON DELETE CASCADE,
  parent_id UUID,
  fractional_index TEXT NOT NULL,
  title TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted BOOLEAN NOT NULL
);
CREATE INDEX threads_parent_id_idx ON threads(parent_id);
CREATE INDEX threads_fractional_index_idx ON threads(fractional_index);
ALTER TABLE threads ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS cards (
  id UUID PRIMARY KEY,
  author UUID REFERENCES users(id),
  pot_id UUID REFERENCES pots(id) ON DELETE CASCADE,
  thread_id UUID NOT NULL,
  fractional_index TEXT NOT NULL,
  content TEXT NOT NULL,
  last_materialized TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted BOOLEAN NOT NULL
);
CREATE INDEX cards_thread_id_idx ON cards(thread_id);
CREATE INDEX cards_fractional_index_idx ON cards(fractional_index);
ALTER TABLE cards ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS card_ydoc_updates (
  id UUID PRIMARY KEY,
  card_id UUID REFERENCES cards(id) NOT NULL,
  data BYTEA NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
ALTER TABLE card_ydoc_updates ENABLE ELECTRIC;
