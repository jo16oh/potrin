import { describe, expect, test } from "vitest";
import { createThread } from "./thread";
import { uuidv7 } from "uuidv7";
import { ELECTRIC_TEST } from "$lib/DataAccess/electric.test";

describe("thread", () => {
  test("create thread", async () => {
    const injectedCreateThread = createThread.inject({
      ELECTRIC: ELECTRIC_TEST,
    });
    const id = uuidv7();
    const result = await injectedCreateThread({ id: id });
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
