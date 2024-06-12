import { describe, expect, test } from "vitest";
import { Card } from "./card";
import { uuidv7 } from "uuidv7";
import { ELECTRIC_TEST } from "$lib/DataAccess/electric.test";

describe("card", async () => {
  const injectedCreateCard = Card.create.inject({ ELECTRIC: ELECTRIC_TEST });
  const injectedUpdateCard = Card.update.inject({ ELECTRIC: ELECTRIC_TEST });
  const id = uuidv7();
  test("create card", async () => {
    const result = await injectedCreateCard({ id: id });
    result._unsafeUnwrap({ withStackTrace: true });
  });
  test("update thread", async () => {
    const result = await injectedUpdateCard(id, { content: "updated" });
    result._unsafeUnwrap({ withStackTrace: true });
  });
});
