import {
  pgTable,
  text,
  timestamp,
  uuid,
  index,
  boolean,
} from "drizzle-orm/pg-core";

export const card = pgTable(
  "cards",
  {
    id: uuid("id").primaryKey(),
    thread_id: uuid("thread_id").notNull(),
    fractional_index: text("fractional_index").notNull(),
    content: text("content").notNull(),
    created_at: timestamp("created_at").notNull(),
    updated_at: timestamp("updated_at").notNull(),
    deleted: boolean("deleted").notNull(),
  },
  (cards) => {
    return {
      thread_idx: index("cards_thread_id_idx").on(cards.thread_id),
      cards_fractional_index_idx: index("cards_fractional_index_idx").on(
        cards.fractional_index,
      ),
    };
  },
);

export const thread = pgTable(
  "threads",
  {
    id: uuid("id").primaryKey(),
    parent_id: uuid("parent_id"),
    fractional_index: text("fractional_index").notNull(),
    title: text("title").notNull(),
    created_at: timestamp("created_at").notNull(),
    updated_at: timestamp("updated_at").notNull(),
    deleted: boolean("deleted").notNull(),
  },
  (threads) => {
    return {
      parent_thread_idx: index("threads_parent_id_idx").on(threads.parent_id),
      threads_fractional_index_idx: index("threads_fractional_index_idx").on(
        threads.fractional_index,
      ),
    };
  },
);
