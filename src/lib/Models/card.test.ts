import { describe, expect, test } from "vitest";
import { createCard } from "./card";
import { uuidv7 } from "uuidv7";
import { ELECTRIC_TEST } from "$lib/DataAccess/electric.test";

describe("card", async () => {
  test("create card", async () => {
    const injectedCreateCard = createCard.inject({ ELECTRIC: ELECTRIC_TEST });
    const id = uuidv7();
    const result = await injectedCreateCard({ id: id });
    result.match(
      (res) => {
        expect(res.id).toBe(id);
      },
      (err) => {
        throw err;
      },
    );
  });
});
