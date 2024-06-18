// import { describe, test } from "vitest";
// import { Thread } from "./thread";
// import { uuidv7 } from "uuidv7";
// import { ELECTRIC_TEST } from "$lib/TestUtils/ELECTRIC_TEST";
//
// describe.skip("thread", async () => {
//   const injectedCreateThread = Thread.create.inject({
//     ELECTRIC: ELECTRIC_TEST,
//   });
//   const injectedUpdateThread = Thread.update.inject({
//     ELECTRIC: ELECTRIC_TEST,
//   });
//   const id = uuidv7();
//
//   test("create thread", async () => {
//     const result = await injectedCreateThread({ id: id });
//     result._unsafeUnwrap({ withStackTrace: true });
//   });
//   test("update thread", async () => {
//     const result = await injectedUpdateThread(id, { title: "updated" });
//     result._unsafeUnwrap({ withStackTrace: true });
//   });
// });
