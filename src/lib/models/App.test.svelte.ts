import { expect, test } from "vitest";
import { App } from "./App.svelte";
import { applyPatch, compare } from "fast-json-patch";
import { tick } from "svelte";
import { deepCloneOwnProperties } from "$lib/utils";
import * as Y from "yjs";

test("app", async () => {
  const ops = [];
  $effect.root(() => {
    let app = App.new({
      clientId: "",
      user: { id: "", name: "bob" },
      pots: {},
      setting: { levenshteinDistance: 0 },
    });

    let prev: App | undefined;

    $effect(() => {
      const current = $state.snapshot(app);
      if (prev) {
        ops.push(compare(prev, current));
      }
      prev = current;
    });

    setTimeout(async () => {
      app.user!.name = "alice";
      await tick();
      expect(ops.length).toBe(1);
      console.log(ops[0]);
    }, 0);
  });
});
