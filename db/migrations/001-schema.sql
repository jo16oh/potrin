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
CREATE INDEX threads_title_idx ON threads(title);
CREATE INDEX threads_parent_id_idx ON threads(parent_id);
ALTER TABLE threads ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS thread_checkpoints (
  id UUID PRIMARY KEY,
  thread_id UUID REFERENCES threads(id) NOT NULL,
  fractional_index TEXT NOT NULL,
  title TEXT NOT NULL
);
CREATE INDEX thread_checkpoints_thread_id_idx ON thread_checkpoints(thread_id);
ALTER TABLE thread_checkpoints ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS cards (
  id UUID PRIMARY KEY,
  author UUID REFERENCES users(id),
  pot_id UUID REFERENCES pots(id) ON DELETE CASCADE,
  thread_id UUID NOT NULL,
  fractional_index TEXT NOT NULL,
  content TEXT NOT NULL,
  last_materialized_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted BOOLEAN NOT NULL
);
CREATE INDEX cards_thread_id_idx ON cards(thread_id);
ALTER TABLE cards ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS card_ydoc_updates (
  id UUID PRIMARY KEY,
  card_id UUID REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
  data BYTEA NOT NULL,
  checkpoint BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX card_ydoc_updates_card_id_idx ON card_ydoc_updates(card_id);
ALTER TABLE card_ydoc_updates ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS card_checkpoints (
  id UUID PRIMARY KEY,
  card_id UUID REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
  ydoc_id UUID REFERENCES card_ydoc_updates(id) ON DELETE CASCADE NOT NULL,
  fractional_index TEXT NOT NULL,
  content TEXT NOT NULL
);
CREATE INDEX card_checkpoints_card_id_idx ON card_checkpoints(card_id);
ALTER TABLE card_checkpoints ENABLE ELECTRIC;


CREATE TABLE IF NOT EXISTS thread_card_checkpoints (
  thread_checkpoint_id UUID REFERENCES thread_checkpoints(id) ON DELETE CASCADE NOT NULL,
  card_checkpoint_id UUID REFERENCES card_checkpoints(id) ON DELETE CASCADE NOT NULL,
  PRIMARY KEY (thread_checkpoint_id, card_checkpoint_id)
);
CREATE INDEX thread_card_checkpoints_thread_idx ON thread_card_checkpoints(thread_checkpoint_id);
CREATE INDEX thread_card_checkpoints_card_idx ON thread_card_checkpoints(card_checkpoint_id);
ALTER TABLE thread_card_checkpoints ENABLE ELECTRIC;
