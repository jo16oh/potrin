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
    thread: uuid("thread").notNull(),
    fractional_index: text("fractional_index").notNull(),
    content: text("content"),
    created_at: timestamp("created_at").notNull(),
    updated_at: timestamp("updated_at").notNull(),
    deleted: boolean("deleted").notNull(),
  },
  (cards) => {
    return {
      thread_idx: index("thread_idx").on(cards.thread),
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
    parent_thread: uuid("parent_thread"),
    fractional_index: text("fractional_index").notNull(),
    title: text("title"),
    created_at: timestamp("created_at").notNull(),
    updated_at: timestamp("updated_at").notNull(),
    deleted: boolean("deleted").notNull(),
  },
  (threads) => {
    return {
      parent_thread_idx: index("parent_thread_idx").on(threads.parent_thread),
      threads_fractional_index_idx: index("threads_fractional_index_idx").on(
        threads.fractional_index,
      ),
    };
  },
);
