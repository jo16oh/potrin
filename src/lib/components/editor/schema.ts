import { Extension, Node } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph as ParagraphSchema } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import Collabolation from "@tiptap/extension-collaboration";
import type { Outline } from "$lib/models/Outline.svelte";
import { Plugin, PluginKey } from "@tiptap/pm/state";
// @ts-expect-error workaround for https://github.com/sveltejs/kit/issues/7805
import { Paragraph } from "$lib/models/Paragraph.svelte?";
import type { Paragraph as ParagraphModel } from "$lib/models/Paragraph.svelte";
import { generateKeyBetween } from "fractional-indexing-jittered";

const OutlineDocument = Node.create({
  name: "doc",
  topNode: true,
  content: "block",
});

export const OutlineSchema = [OutlineDocument, ParagraphSchema, Text];

export const createParagraphSchema = async (
  paragraph: ParagraphModel,
  updateFocusPosition: (pos: FocusPosition) => void,
) => {
  const ydoc = await paragraph.ydoc();

  return [
    Document,
    ParagraphSchema,
    Text,
    Collabolation.configure({
      fragment: ydoc.getXmlFragment("doc"),
    }),
    Extension.create({
      name: "KeydownHandler",
      addProseMirrorPlugins() {
        return [
          new Plugin({
            key: new PluginKey("KeydownHandler"),
            props: {
              handleKeyDown(view, event) {
                const editorRect = view.dom.getBoundingClientRect();
                const cursorRect = view.coordsAtPos(view.state.selection.from);
                const REM = 16;

                if (event.key === "Enter") {
                  const currentNode = view.state.selection.$from.node();
                  const isDocEmpty = view.state.doc.content.size <= 2;
                  const isCurrentNodeEmpty =
                    currentNode.content.size === 0 &&
                    currentNode.textContent.length === 0;

                  if (isDocEmpty || !isCurrentNodeEmpty) return false;

                  const { $from } = view.state.selection;
                  const node = $from.node();
                  const pos = $from.before();
                  view.dispatch(view.state.tr.delete(pos, pos + node.nodeSize));

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

                  return true;
                }

                if (
                  event.key === "Backspace" &&
                  view.state.selection.from === 2
                ) {
                  const yMap = ydoc.getMap("potrin");
                  yMap.set("deleted", true);

                  const paragraphBeforeIndex =
                    paragraph.outline!.paragraphs.findIndex(
                      (p) => p.id === paragraph.id,
                    ) - 1;

                  const paragraphBefore =
                    paragraph.outline!.paragraphs[paragraphBeforeIndex];

                  paragraph.outline?.removeParagraph(paragraph);

                  updateFocusPosition({
                    id: paragraphBefore
                      ? paragraphBefore.id
                      : paragraph.outline
                        ? paragraph.outline.id
                        : null,

                    position: "end",
                  });

                  return false;
                }

                if (
                  event.key === "ArrowUp" &&
                  cursorRect.top - editorRect.top < REM
                ) {
                  const paragraphBefore =
                    paragraph.outline?.paragraphs[
                      paragraph.outline?.paragraphs.findIndex(
                        (p) => p.id === paragraph.id,
                      ) - 1
                    ];

                  updateFocusPosition({
                    id: paragraphBefore
                      ? paragraphBefore.id
                      : paragraph.outline!.id,
                    position: "end",
                  });

                  return false;
                }

                if (
                  event.key === "ArrowDown" &&
                  editorRect.bottom - cursorRect.bottom < REM
                ) {
                  const paragraphAfter =
                    paragraph.outline?.paragraphs[
                      paragraph.outline?.paragraphs.findIndex(
                        (p) => p.id === paragraph.id,
                      ) + 1
                    ];

                  if (paragraphAfter) {
                    updateFocusPosition({
                      id: paragraphAfter.id,
                      position: "start",
                    });
                  }

                  return false;
                }

                return false;
              },
            },
          }),
        ];
      },
    }),
  ];
};

export const createOutlineSchema = async (
  outline: Outline,
  updateFocusPosition: (pos: FocusPosition) => void,
) => {
  const ydoc = await outline.ydoc();

  return [
    OutlineDocument,
    ParagraphSchema,
    Text,
    Collabolation.configure({
      fragment: ydoc.getXmlFragment("doc"),
    }),
    Extension.create({
      name: "KeydownHandler",
      addProseMirrorPlugins() {
        return [
          new Plugin({
            key: new PluginKey("KeydownHandler"),
            props: {
              handleKeyDown(view, event) {
                const editorRect = view.dom.getBoundingClientRect();
                const cursorRect = view.coordsAtPos(view.state.selection.from);
                const REM = 16;

                if (event.key === "Enter") {
                  const newParagraph = Paragraph.new(
                    outline,
                    generateKeyBetween(null, null),
                  );

                  outline.insertParagraph(newParagraph);

                  updateFocusPosition({
                    id: newParagraph.id,
                    position: "start",
                  });

                  return true;
                }

                if (
                  event.key === "Backspace" &&
                  view.state.selection.from === 2
                ) {
                  return false;
                }

                if (
                  event.key === "ArrowUp" &&
                  cursorRect.top - editorRect.top < REM
                ) {
                  // const paragraphBefore =
                  //   paragraph.outline!.paragraphs[
                  //     paragraph.outline!.paragraphs.findIndex(
                  //       (p) => p.id === paragraph.id,
                  //     ) - 1
                  //   ];
                  //
                  // updateFocusPosition({
                  //   id: paragraphBefore
                  //     ? paragraphBefore.id
                  //     : paragraph.outline!.id,
                  //   position: "end",
                  // });

                  return false;
                }

                if (
                  event.key === "ArrowDown" &&
                  editorRect.bottom - cursorRect.bottom < REM
                ) {
                  const paragraphAfter = outline.paragraphs[0];

                  if (paragraphAfter) {
                    updateFocusPosition({
                      id: paragraphAfter.id,
                      position: "start",
                    });
                  }

                  return false;
                }

                return false;
              },
            },
          }),
        ];
      },
    }),
  ];
};
