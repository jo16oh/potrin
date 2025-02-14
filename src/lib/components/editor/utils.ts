import type {
  JSONContent,
  FocusPosition as TiptapFocusPosition,
} from "@tiptap/core";
import type { Schema } from "@tiptap/pm/model";
import {
  prosemirrorJSONToYXmlFragment,
  yXmlFragmentToProseMirrorRootNode,
} from "y-prosemirror";
import * as Y from "yjs";

export type EditorFocusPosition = TiptapFocusPosition;

export type FocusPosition = {
  id: string | null;
  position: EditorFocusPosition;
};

export function insertJSONContentsToYXMLFragment(
  content: JSONContent[],
  schema: Schema,
  fragment: Y.XmlFragment,
  ydoc: Y.Doc,
  appendToLastNode?: boolean,
) {
  const doc = {
    type: "doc",
    content: content,
  };

  const fragmentToInsert = new Y.Doc().getXmlFragment();
  prosemirrorJSONToYXmlFragment(schema, doc, fragmentToInsert);

  const lastNode = fragment.get(fragment.length - 1);

  ydoc.transact(() => {
    fragmentToInsert.forEach((e, i) => {
      if (i === 0 && e.length === 0) {
        return;
      } else if (
        appendToLastNode &&
        i === 0 &&
        lastNode instanceof Y.XmlElement &&
        e instanceof Y.XmlElement &&
        lastNode.nodeName === e.nodeName
      ) {
        e.forEach((e) => {
          lastNode.insert(lastNode.length, [e.clone()]);
        });
      } else {
        fragment.insert(fragment.length, [e.clone()]);
      }
    });
  });
}

export function countSizeOfYXMLFragment(
  fragment: Y.XmlFragment,
  schema: Schema,
): number {
  const doc = yXmlFragmentToProseMirrorRootNode(fragment, schema);
  return doc.nodeSize;
}
