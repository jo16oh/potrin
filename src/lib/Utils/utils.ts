export const sql = String.raw;

export function uint8ArrayToBase64(bytes: Uint8Array) {
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
