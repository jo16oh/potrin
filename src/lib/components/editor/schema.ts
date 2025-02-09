import { Node } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";

const OutlineDocument = Node.create({
  name: "doc",
  topNode: true,
  content: "block",
});

export const OutlineSchema = [OutlineDocument, Paragraph, Text];

export const ParagraphSchema = [Document, Paragraph, Text];
