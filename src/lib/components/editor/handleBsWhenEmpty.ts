import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { Paragraph } from "$lib/models/Paragraph.svelte";
import type { FocusPosition } from "./utils";

export function handleBsWhenEmpty(
  paragraph: Paragraph,
  updateFocusPosition: (pos: FocusPosition) => void,
) {
  return Extension.create({
    name: "handleBSwhenEmpty",

    addProseMirrorPlugins() {
      return [
        new Plugin({
          key: new PluginKey("handleBsWhenEmpty"),
          props: {
            handleKeyDown(view, event) {
              if (event.key !== "Backspace") return false;
              if (view.state.doc.textContent) return false;
              event.preventDefault();

              (async () => {
                const ydoc = await paragraph.ydoc();
                const yMap = ydoc.getMap("potrin");
                yMap.set("deleted", false);

                const paragraphBeforeIndex =
                  paragraph.outline!.paragraphs.findIndex(
                    (p) => p.id === paragraph.id,
                  ) - 1;

                const paragraphBefore =
                  paragraph.outline!.paragraphs[paragraphBeforeIndex];

                console.log(paragraphBefore);

                paragraph.outline?.removeParagraph(paragraph);

                updateFocusPosition({
                  id: paragraphBefore
                    ? paragraphBefore.id
                    : paragraph.outline
                      ? paragraph.outline.id
                      : null,

                  position: "end",
                });
              })();

              // ProseMirrorの処理を止めるためにtrueを返す
              return true;
            },
          },
        }),
      ];
    },
  });
}
