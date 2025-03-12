import { unwrap } from "$lib/utils";
import {
  commands,
  events,
  type ParagraphPositionIndex,
} from "generated/tauri-commands";
import { Outline } from "./Outline.svelte";
import { Paragraph } from "./Paragraph.svelte";
import type { View } from "./Workspace.svelte";
import { watch } from "runed";
import { getCurrent } from "@tauri-apps/api/webviewWindow";
import { debounce } from "es-toolkit";

export type SearchResultItem = {
  outline: Outline;
  paragraphs: Paragraph[];
};

export class Search {
  #view: View<"search">;
  #query = $state("");
  #outline: Outline | null = $state(null);
  result: SearchResultItem[] = $state([]);
  paragraphPositionIndex: ParagraphPositionIndex = $state({});
  #cleanup: (() => void) | undefined;

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

    const cleanupEffect = $effect.root(() => {
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

      watch(
        () => [this.#query, this.#view.orderBy, this.#outline],
        () => {
          if (this.#query.length === 0) {
            this.result = [];
            return;
          }

          this.reload();
        },
      );
    });

    (async () => {
      const debouncedReload = debounce(this.reload, 200);

      const cleanupOutlineEvent = await events
        .outlineChange(getCurrent())
        .listen(debouncedReload);

      const cleanupParagraphEvent = await events
        .paragraphChange(getCurrent())
        .listen(debouncedReload);

      this.#cleanup = () => {
        cleanupEffect();
        cleanupOutlineEvent();
        cleanupParagraphEvent();
      };
    })();
  }

  reload = async () => {
    const scope = this.#outline
      ? (await this.#outline.path).map((l) => l.id)
      : null;

    commands
      .search(this.#query, scope, { updatedAt: "desc" }, 0, 100)
      .then(unwrap)
      .then(([rawOutlines, rawParagraphs, resultOrder, paragraphPosition]) => {
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

        const orderMap = new Map(resultOrder.map((id, i) => [id, i]));

        const result = Array.from(outlines.values()).map((o) => {
          return {
            outline: o,
            paragraphs: paragraphs.get(o.id) ?? [],
          };
        });

        if (this.#view.orderBy !== "relevance") {
          const order =
            "createdAt" in this.#view.orderBy
              ? this.#view.orderBy.createdAt
              : this.#view.orderBy.updatedAt;

          if (order === "asc") {
            result.sort((a, b) => {
              return (
                Math.max(
                  ...[
                    orderMap.get(b.outline.id),
                    ...b.paragraphs.map((p) => orderMap.get(p.id)),
                  ].filter((i) => i !== undefined),
                ) -
                Math.max(
                  ...[
                    orderMap.get(a.outline.id),
                    ...a.paragraphs.map((p) => orderMap.get(p.id)),
                  ].filter((i) => i !== undefined),
                )
              );
            });
          } else {
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
          }
        }

        this.result = result;
        this.paragraphPositionIndex = paragraphPosition;
      });
  };

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
