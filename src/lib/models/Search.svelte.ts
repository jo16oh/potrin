import { unwrap } from "$lib/utils";
import {
  commands,
  type ParagraphPositionIndex,
} from "generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import { Paragraph } from "./Paragraph.svelte";
import type { View } from "./Workspace.svelte";
import { watch } from "runed";

export type SearchResultItem = {
  outline: Outline;
  paragraphs: Paragraph[];
};

export class Search {
  #view: View<"search">;
  #query = $state("");
  #outline: Outline | null = $state(null);
  result: Promise<SearchResultItem[]> = $state(new Promise(() => {}));
  paragraphPositionIndex: ParagraphPositionIndex = $state({});
  #cleanup: () => void | undefined;

  static async init(view: View<"search">) {
    const outline = view.scope
      ? await commands
          .fetchOutlineWithPathById(view.scope)
          .then(unwrap)
          .then((o) => (o ? Outline.from(o) : null))
      : null;

    return new Search(view, outline);
  }

  private constructor(view: View<"search">, outline: Outline | null) {
    this.#view = view;
    this.#query = view.query;
    this.#outline = outline;

    this.#cleanup = $effect.root(() => {
      watch(
        () => view.scope,
        () => {
          (async () => {
            this.#outline = view.scope
              ? await commands
                  .fetchOutlineWithPathById(view.scope)
                  .then(unwrap)
                  .then((o) => (o ? Outline.from(o) : null))
              : null;
          })();
        },
        { lazy: true },
      );

      $effect(() => {
        if (this.#query.length === 0) {
          this.result = Promise.resolve([]);
          return;
        }

        (async () => {
          const outline = this.#outline;
          const path = outline ? await outline.path : null;
          const scope = path ? path.map((l) => l.id) : null;

          commands
            .search(this.#query, scope, { updatedAt: "desc" }, 0, 100)
            .then(unwrap)
            .then(
              ([
                rawOutlines,
                rawParagraphs,
                searchResults,
                paragraphPosition,
              ]) => {
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
                this.paragraphPositionIndex = paragraphPosition;
              },
            );
        })();
      });
    });
  }

  get query() {
    return this.#query;
  }

  set query(value: string) {
    this.#view.query = this.#query = value;
  }

  get path() {
    return this.#outline?.path;
  }

  cleanup = () => {
    this.#cleanup?.();
  };
}
