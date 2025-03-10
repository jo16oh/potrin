import { unwrap } from "$lib/utils";
import { commands } from "generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import { Paragraph } from "./Paragraph.svelte";
import type { View } from "./Workspace.svelte";

type Item = {
  outline: Outline;
  paragraphs: Paragraph[];
};

export class Search {
  #view: View<"search">;
  #query = $state("");
  result: Promise<Item[]> = $state(new Promise(() => {}));
  #cleanup: () => void | undefined;

  constructor(view: View<"search">) {
    this.#view = view;
    this.#query = view.query;

    this.#cleanup = $effect.root(() => {
      $effect(() => {
        if (this.#query.length === 0) {
          this.result = Promise.resolve([]);
          return;
        }

        const scope = this.#view.path ? this.#view.path.map((p) => p.id) : null;

        commands
          .search(this.#query, scope, { updatedAt: "desc" }, 0, 100)
          .then(unwrap)
          .then(([rawOutlines, rawParagraphs, searchResults]) => {
            const outlines = new Map(
              rawOutlines.map((o) => [o.id, Outline.from(o)]),
            );

            const paragraphs = Map.groupBy(
              rawParagraphs
                .map((p) => {
                  const o = outlines.get(p.outlineId);
                  return o ? Paragraph.from(p, o) : null;
                })
                .filter((p) => p !== null),
              (p) => p.outlineId,
            );

            const orderMap = new Map(searchResults.map((id, i) => [id, i]));

            const result = Array.from(outlines.values()).map((o) => {
              return {
                outline: o,
                paragraphs: paragraphs.get(o.id) ?? [],
              };
            });

            result.sort((a, b) => {
              return (
                Math.min(
                  ...[
                    orderMap.get(a.outline.id),
                    ...a.paragraphs.map((p) => orderMap.get(p.id)),
                  ].filter((i) => i !== undefined),
                ) -
                Math.min(
                  ...[
                    orderMap.get(b.outline.id),
                    ...b.paragraphs.map((p) => orderMap.get(p.id)),
                  ].filter((i) => i !== undefined),
                )
              );
            });

            this.result = Promise.resolve(result);
          });
      });
    });
  }

  get query() {
    return this.#query;
  }

  set query(value: string) {
    this.#view.query = value;
    this.#query = value;
  }

  cleanup = () => {
    this.#cleanup?.();
  };
}
