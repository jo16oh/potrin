import { Extension, Node } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph as ParagraphSchema } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import Collabolation from "@tiptap/extension-collaboration";
import {
  countSizeOfYXMLFragment as sizeOfYXMLFragment,
  insertJSONContentsToYXMLFragment,
  type FocusPosition,
} from "./utils";
import type { Outline } from "$lib/models/Outline.svelte";
import { Plugin, PluginKey } from "@tiptap/pm/state";
// @ts-expect-error workaround for https://github.com/sveltejs/kit/issues/7805
import { Paragraph } from "$lib/models/Paragraph.svelte?";
import type { Paragraph as ParagraphModel } from "$lib/models/Paragraph.svelte";
import { generateKeyBetween } from "fractional-indexing-jittered";
import * as Y from "yjs";

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
  const yDoc = await paragraph.ydoc();
  const yMap = yDoc.getMap("potrin");
  const fragment = yMap.get("doc") as Y.XmlFragment;

  return [
    Document,
    ParagraphSchema,
    Text,
    Collabolation.configure({
      fragment: fragment,
    }),
    Extension.create({
      name: "KeydownHandler",
      addProseMirrorPlugins() {
        return [
          new Plugin({
            key: new PluginKey("KeydownHandler"),
            props: {
              handleKeyDown(view, event) {
                const state = view.state;
                const doc = state.doc;
                const selection = state.selection;
                const { $from } = state.selection;

                const REM = 16;

                const isDocEmpty = doc.content.size <= 2;
                const isCursorAtDocStart = selection.from === 1;
                const isPreviousNodeEmpty = !$from.nodeBefore;
                const isCursorAtNodeStart = $from.parentOffset === 0;

                if (event.key === "Enter") {
                  if (isCursorAtDocStart) return true;
                  if (!isPreviousNodeEmpty || !isCursorAtNodeStart)
                    return false;

                  view.dom.blur();

                  const end = doc.content.size;

                  const content = doc
                    .slice(selection.from, end)
                    .toJSON().content;

                  view.dispatch(state.tr.delete($from.pos - 1, end));

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
                  ) as ParagraphModel;

                  paragraph.outline?.insertParagraph(newParagraph);

                  newParagraph.ydoc().then((ydoc) => {
                    const fragment = ydoc
                      .getMap("potrin")
                      .get("doc") as Y.XmlFragment;

                    insertJSONContentsToYXMLFragment(
                      content,
                      state.schema,
                      fragment,
                      ydoc,
                    );

                    updateFocusPosition({
                      id: newParagraph.id,
                      position: "start",
                    });
                  });

                  return true;
                }

                if (event.key === "Backspace" && selection.from === 1) {
                  const paragraphBeforeIndex =
                    paragraph.outline!.paragraphs.findIndex(
                      (p) => p.id === paragraph.id,
                    ) - 1;

                  if (paragraphBeforeIndex !== -1) {
                    const paragraphBefore =
                      paragraph.outline!.paragraphs[paragraphBeforeIndex];

                    paragraphBefore?.ydoc().then((ydocBefore) => {
                      const fragmentBefore = ydocBefore
                        .getMap("potrin")
                        .get("doc") as Y.XmlFragment;

                      const sizeOfFragmentBefore = sizeOfYXMLFragment(
                        fragmentBefore,
                        state.schema,
                      );

                      const content = doc.toJSON().content;

                      insertJSONContentsToYXMLFragment(
                        content,
                        state.schema,
                        fragmentBefore,
                        ydocBefore,
                        false,
                      );

                      paragraph.outline?.removeParagraph(paragraph);
                      yDoc.transact(() => {
                        yMap.set("deleted", true);
                        fragment.delete(1, fragment.length - 1);
                      });

                      updateFocusPosition({
                        id: paragraphBefore
                          ? paragraphBefore.id
                          : paragraph.outline
                            ? paragraph.outline.id
                            : null,
                        position: sizeOfFragmentBefore - 1,
                      });
                    });
                  } else if (isDocEmpty) {
                    yDoc.transact(() => {
                      yMap.set("deleted", true);
                    });

                    paragraph.outline?.removeParagraph(paragraph);

                    updateFocusPosition({
                      id: paragraph.outline?.id ?? null,
                      position: "end",
                    });
                  }

                  return false;
                }

                if (event.key === "ArrowUp") {
                  const editorRect = view.dom.getBoundingClientRect();
                  const cursorRect = view.coordsAtPos(selection.from);

                  if (cursorRect.top - editorRect.top < REM) {
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
                }

                if (event.key === "ArrowDown") {
                  const editorRect = view.dom.getBoundingClientRect();
                  const cursorRect = view.coordsAtPos(selection.from + 1);

                  if (editorRect.bottom - cursorRect.bottom < REM) {
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
  const fragment = ydoc.getMap("potrin").get("doc") as Y.XmlFragment;

  return [
    OutlineDocument,
    ParagraphSchema,
    Text,
    Collabolation.configure({
      fragment: fragment,
    }),
    Extension.create({
      name: "KeydownHandler",
      addProseMirrorPlugins() {
        return [
          new Plugin({
            key: new PluginKey("KeydownHandler"),
            props: {
              handleKeyDown(view, event) {
                const state = view.state;
                const selection = state.selection;

                const editorRect = view.dom.getBoundingClientRect();
                const cursorRect = view.coordsAtPos(selection.from);
                const REM = 16;

                if (event.key === "Enter") {
                  const newParagraph = Paragraph.new(
                    outline,
                    generateKeyBetween(
                      null,
                      outline.paragraphs[0]?.fractionalIndex ?? null,
                    ),
                  );

                  outline.insertParagraph(newParagraph);

                  updateFocusPosition({
                    id: newParagraph.id,
                    position: "start",
                  });

                  return true;
                }

                if (event.key === "Backspace" && selection.from === 2) {
                  return false;
                }

                if (
                  event.key === "ArrowUp" &&
                  cursorRect.top - editorRect.top < REM
                ) {
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
