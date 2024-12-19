import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { uuidv7obj } from "uuidv7";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

declare const _brand: unique symbol;

export type Brand<B> = {
  readonly [_brand]: B;
};

export function uint8ArrayToBase64(bytes: Uint8Array): string {
  let binaryString = "";

  for (const byte of bytes) {
    binaryString += String.fromCharCode(byte);
  }

  return btoa(binaryString);
}

export function base64ToUint8Array(base64: string): Uint8Array {
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);

  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }

  return bytes;
}

export function uuidv7() {
  return uint8ArrayToBase64(uuidv7obj().bytes);
}

export function insertToFractionalIndexArray<
  T extends { fractionalIndex: string },
>(arr: T[], item: T): T[] {
  if (arr.length === 0) {
    arr.push(item);
  }

  let low = 0;
  let high = arr.length;

  while (low < high) {
    const mid = (low + high) >>> 1;
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
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
