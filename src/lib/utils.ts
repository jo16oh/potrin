import { uuidv7obj } from "uuidv7";
// @ts-expect-error no type declaration
import { toBytes, toBase64 } from "fast-base64/js";
// @ts-expect-error no type declaration
import { toUrl, fromUrl } from "fast-base64/url";
import type { Result } from "../generated/tauri-commands";

declare const _brand: unique symbol;

export type Brand<B> = {
  readonly [_brand]: B;
};

export function uint8ArrayToBase64URL(bytes: Uint8Array): string {
  return toUrl(toBase64(bytes));
}

export function base64URLToUint8Array(base64: string): Uint8Array {
  return toBytes(fromUrl(base64));
}

export function uuidv7() {
  return uint8ArrayToBase64URL(uuidv7obj().bytes);
}

export function insertToFractionalIndexedArray<
  T extends { fractionalIndex: string },
>(arr: T[], item: T): T[] {
  if (arr.length === 0) {
    arr.push(item);
  }

  let low = 0;
  let high = arr.length;

  while (low < high) {
    const mid = (low + high) >>> 1;
    if (arr[mid]!.fractionalIndex < item.fractionalIndex) low = mid + 1;
    else high = mid;
  }

  arr.splice(low, 0, item);

  return arr;
}

export function byFractionalIndex<T extends { fractionalIndex: string }>(
  a: T,
  b: T,
): number {
  if (a.fractionalIndex < b.fractionalIndex) {
    return -1;
  } else if (a.fractionalIndex > b.fractionalIndex) {
    return 1;
  } else {
    return 0;
  }
}

export type ExcludeMethods<T> = {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  [K in keyof T]: T[K] extends Function
    ? never
    : T[K] extends object
      ? ExcludeMethods<T[K]>
      : T[K];
};

export function unwrap<T, E>(result: Result<T, E>) {
  if (result.status === "ok") {
    return result.data;
  } else {
    throw result.error;
  }
}
