import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { FocusPosition } from "./utils";
import { generateKeyBetween } from "fractional-indexing-jittered";
// @ts-expect-error workaround for https://github.com/sveltejs/kit/issues/7805
import { Paragraph } from "$lib/models/Paragraph.svelte?";
import type { Outline } from "$lib/models/Outline.svelte";
import type { Paragraph as ParagraphModel } from "$lib/models/Paragraph.svelte";

export function handleEnterLastLineParagraph(
  paragraph: ParagraphModel,
  updateFocusPosition: (pos: FocusPosition) => void,
) {
  return Extension.create({
    name: "handleEnterLastLine",

    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: new PluginKey("handleEnterLastLine"),
          props: {
            handleKeyDown(view, event) {
              if (event.key !== "Enter") return false;
              if (view.state.doc.lastChild?.textContent) return false;
              if (view.state.selection.head !== view.state.doc.content.size - 1)
                return false;
              if (view.state.doc.content.size === 2) return true;

              event.preventDefault();

              // 最後のノードを削除
              const lastNodePos =
                view.state.doc.content.size -
                view.state.doc.lastChild!.nodeSize!;
              const tr = view.state.tr.delete(
                lastNodePos,
                view.state.doc.content.size,
              );
              view.dispatch(tr);

              const paragraphAfter =
                paragraph.outline!.paragraphs[
                  paragraph.outline!.paragraphs.findIndex(
                    (p) => p.id === paragraph.id,
                  ) + 1
                ];

              const newParagraph = Paragraph.new(
                paragraph.outline!,
                generateKeyBetween(
                  paragraph.fractionalIndex,
                  paragraphAfter ? paragraphAfter.fractionalIndex : null,
                ),
              );

              paragraph.outline?.insertParagraph(newParagraph);

              updateFocusPosition({
                id: newParagraph.id,
                position: "start",
              });

              // ProseMirrorの処理を止めるためにtrueを返す
              return true;
            },
          },
        }),
      ];
    },
  });
}

export function handleEnterLastLineOutline(
  outline: Outline,
  updateFocusPosition: (pos: FocusPosition) => void,
) {
  return Extension.create({
    name: "handleEnterLastLine",

    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: new PluginKey("handleEnterLastLine"),
          props: {
            handleKeyDown(_view, event) {
              if (event.key !== "Enter") return false;
              // if (view.state.doc.lastChild?.textContent) return false;
              // if (view.state.selection.head !== view.state.doc.content.size - 1)
              //   return false;
              // if (view.state.doc.content.size === 2) return true;

              event.preventDefault();

              if (outline.paragraphs[0]) {
                updateFocusPosition({
                  id: outline.paragraphs[0].id,
                  position: "end",
                });
              } else {
                const newParagraph = Paragraph.new(
                  outline,
                  generateKeyBetween(null, null),
                );

                outline.insertParagraph(newParagraph);

                updateFocusPosition({
                  id: newParagraph.id,
                  position: "start",
                });
              }

              // ProseMirrorの処理を止めるためにtrueを返す
              return true;
            },
          },
        }),
      ];
    },
  });
}
