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
    thread: uuid("thread"),
    prev_card: uuid("prev_card"),
    content: text("content"),
    created_at: timestamp("created_at"),
    updated_at: timestamp("updated_at"),
  },
  (cards) => {
    return {
      thread_idx: index("thread_idx").on(cards.thread),
      prev_card_idx: index("prev_card_idx").on(cards.prev_card),
    };
  },
);

export const thread = pgTable(
  "threads",
  {
    id: uuid("id").primaryKey(),
    parent_thread: uuid("parent_thread"),
    prev_thread: uuid("prev_thread"),
    title: text("title"),
    created_at: timestamp("created_at"),
    updated_at: timestamp("updated_at"),
    deleted: boolean("deleted"),
  },
  (threads) => {
    return {
      parent_thread_idx: index("parent_thread_idx").on(threads.parent_thread),
      prev_thread_idx: index("prev_thread_idx").on(threads.prev_thread),
    };
  },
);
