import type { FocusPosition as TiptapFocusPosition } from "@tiptap/core";

export type EditorFocusPosition = TiptapFocusPosition;

export type FocusPosition = {
  id: string | null;
  position: EditorFocusPosition;
};
