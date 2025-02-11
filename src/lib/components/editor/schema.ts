import { Node } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import Collabolation from "@tiptap/extension-collaboration";
import {
  handleEnterLastLineParagraph,
  handleEnterLastLineOutline,
} from "./handleEnterLastLine";
import { handleBsWhenEmpty } from "./handleBsWhenEmpty";
import type { FocusPosition } from "./utils";
import type { Paragraph as ParagraphModel } from "$lib/models/Paragraph.svelte";
import type { Outline } from "$lib/models/Outline.svelte";

const OutlineDocument = Node.create({
  name: "doc",
  topNode: true,
  content: "block",
});

export const OutlineSchema = [OutlineDocument, Paragraph, Text];

export const createParagraphSchema = async (
  paragraph: ParagraphModel,
  updateFocusPosition: (pos: FocusPosition) => void,
) => {
  const ydoc = await paragraph.ydoc();

  return [
    Document,
    Paragraph,
    Text,
    Collabolation.configure({
      fragment: ydoc.getXmlFragment("doc"),
    }),
    handleEnterLastLineParagraph(paragraph, updateFocusPosition),
    handleBsWhenEmpty(paragraph, updateFocusPosition),
  ];
};

export const createOutlineSchema = async (
  outline: Outline,
  updateFocusPosition: (pos: FocusPosition) => void,
) => {
  const ydoc = await outline.ydoc();

  return [
    OutlineDocument,
    Paragraph,
    Text,
    Collabolation.configure({
      fragment: ydoc.getXmlFragment("doc"),
    }),
    handleEnterLastLineOutline(outline, updateFocusPosition),
  ];
};
