import * as Y from "yjs";

export type AnyYMapValue =
  | null
  | object
  | boolean
  | string
  | number
  | Uint8Array
  | Y.AbstractType<any>;
