import type { Links, Path, Quote } from "../../generated/tauri-commands";
import type { Outline } from "./Outline.svelte";
import type { Paragraph } from "./Paragraph.svelte";
import * as Y from "yjs";

export class WeakRefMap<K, T extends WeakKey> {
  readonly #map: Map<K, WeakRef<T>> = new Map();
  readonly #finalizationRegistry = new FinalizationRegistry<K>((key) => {
    if (this.#map.get(key)?.deref()) return;
    this.#map.delete(key);
    for (const fn of this.#finalizationHooks.values()) {
      fn(key);
    }
  });
  readonly #finalizationHooks = new Set<(key: K) => void>();

  set(key: K, value: T) {
    this.#map.set(key, new WeakRef(value));
    this.#finalizationRegistry.register(value, key);
  }

  get(key: K): T | undefined {
    const ref = this.#map.get(key);
    if (!ref) return undefined;

    const value = ref.deref();

    if (value) {
      return value;
    } else {
      this.#map.delete(key);
      return undefined;
    }
  }

  delete(key: K): boolean {
    return this.#map.delete(key);
  }

  addHook(hook: (key: K) => void): () => void {
    this.#finalizationHooks.add(hook);
    return () => {
      this.#finalizationHooks.delete(hook);
    };
  }

  get size() {
    return this.#map.size;
  }
}

export class ReversedLinkIndex<T extends { links: Links }> {
  readonly #reversedLinkIndex = new Map<string, Set<string>>();
  readonly #prevLinksMap = new Map<string, Links>();
  readonly #buffer: WeakRefMap<string, T>;

  constructor(buffer: WeakRefMap<string, T>) {
    this.#buffer = buffer;
    this.#buffer.addHook((deletedId) => {
      if (!this.#buffer.get(deletedId)) return;

      const links = this.#prevLinksMap.get(deletedId);
      if (!links) return;

      for (const id_to of Object.keys(links)) {
        const backlinks = this.#reversedLinkIndex.get(id_to);
        backlinks?.delete(deletedId);
        if (backlinks?.size === 0) this.#reversedLinkIndex.delete(id_to);
      }

      this.#prevLinksMap.delete(deletedId);
    });
  }

  set(id_from: string, links: Links) {
    const prevLinks = this.#prevLinksMap.get(id_from);
    const prev = prevLinks
      ? new Set(Object.keys(prevLinks))
      : new Set<string>();
    const current = new Set(Object.keys(links));

    for (const id_to of prev.difference(current)) {
      const backlinks = this.#reversedLinkIndex.get(id_to);
      backlinks?.delete(id_from);
      if (backlinks?.size === 0) this.#reversedLinkIndex.delete(id_to);
    }

    for (const id_to of current.difference(prev)) {
      const backlinks = this.#reversedLinkIndex.get(id_to);
      if (backlinks) {
        backlinks.add(id_from);
      } else {
        this.#reversedLinkIndex.set(id_to, new Set([id_from]));
      }
    }

    this.#prevLinksMap.set(id_from, { ...links });
  }

  get(id_to: string): T[] {
    return (
      this.#reversedLinkIndex
        .get(id_to)
        ?.keys()
        .map((id_from) => this.#buffer.get(id_from))
        .filter((o) => o !== undefined)
        .toArray() ?? []
    );
  }
}

export class DescendantsIndex {
  readonly #descendantsMap = new Map<string, Set<string>>();
  readonly #prevPathMap = new Map<string, Path>();
  readonly #outlineBuffer: WeakRefMap<string, Outline>;
  readonly #paragraphBuffer: WeakRefMap<string, Paragraph>;

  constructor(
    outlineBuffer: WeakRefMap<string, Outline>,
    paragraphBuffer: WeakRefMap<string, Paragraph>,
  ) {
    this.#outlineBuffer = outlineBuffer;
    this.#paragraphBuffer = paragraphBuffer;
    this.#outlineBuffer.addHook(this.#reconcile);
    this.#paragraphBuffer.addHook(this.#reconcile);
  }

  // Using arrow function to fix `this` to the instance,
  // which avoids `this` reference error in FinalizationRegistry
  #reconcile = (deletedId: string) => {
    if (
      !(
        this.#outlineBuffer.get(deletedId) ??
        this.#paragraphBuffer.get(deletedId)
      )
    )
      return;

    const ancestors = this.#prevPathMap.get(deletedId);
    if (!ancestors) return;

    for (const { id } of ancestors) {
      const ancestorsDescendants = this.#descendantsMap.get(id);
      ancestorsDescendants?.delete(deletedId);
    }

    this.#prevPathMap.delete(deletedId);
  };

  set(descendantId: string, path: Path) {
    // Exclude the last element because the last element
    // represents the document itself, not its ancestors
    const currentPath = path.slice(0, -1);

    const prevPath = this.#prevPathMap.get(descendantId);
    const prev = prevPath
      ? new Set(prevPath.map((link) => link.id))
      : new Set<string>();
    const current = new Set(currentPath.map((link) => link.id));

    for (const ancestorId of prev.difference(current)) {
      const descendants = this.#descendantsMap.get(ancestorId);
      descendants?.delete(descendantId);
      if (descendants?.size === 0) this.#descendantsMap.delete(ancestorId);
    }

    for (const ancestorId of current.difference(prev)) {
      const descendants = this.#descendantsMap.get(ancestorId);
      if (descendants) {
        descendants.add(descendantId);
      } else {
        this.#descendantsMap.set(ancestorId, new Set([descendantId]));
      }
    }

    this.#prevPathMap.set(descendantId, currentPath);
  }

  get(ancestorId: string) {
    return (
      this.#descendantsMap
        .get(ancestorId)
        ?.keys()
        .map(
          (id) => this.#outlineBuffer.get(id) ?? this.#paragraphBuffer.get(id),
        )
        .filter((o) => o !== undefined)
        .toArray() ?? []
    );
  }
}

export class ReversedQuoteIndex {
  readonly #reversedQuoteIndex = new Map<string, Set<string>>();
  readonly #prevQuoteMap = new Map<string, Quote>();
  readonly #buffer: WeakRefMap<string, Paragraph>;

  constructor(buffer: WeakRefMap<string, Paragraph>) {
    this.#buffer = buffer;
    this.#buffer.addHook((deletedId) => {
      if (!this.#buffer.get(deletedId)) return;

      const quote = this.#buffer.get(deletedId)?.quote;
      if (!quote) return;

      const backlinks = this.#reversedQuoteIndex.get(quote.id);
      backlinks?.delete(deletedId);
      if (backlinks?.size === 0) this.#reversedQuoteIndex.delete(quote.id);

      this.#prevQuoteMap.delete(deletedId);
    });
  }

  set(id_from: string, quote: Quote | null) {
    const prevQuote = this.#prevQuoteMap.get(id_from);
    if (quote) {
      const backlinks = this.#reversedQuoteIndex.get(quote.id);
      if (backlinks) {
        backlinks.add(id_from);
      } else {
        this.#reversedQuoteIndex.set(quote.id, new Set([id_from]));
      }
      this.#prevQuoteMap.set(id_from, quote);
    } else if (prevQuote) {
      const backlinks = this.#reversedQuoteIndex.get(prevQuote.id);
      backlinks?.delete(id_from);
      if (backlinks?.size === 0) this.#reversedQuoteIndex.delete(prevQuote.id);
      this.#prevQuoteMap.delete(id_from);
    }
  }

  get(id_to: string): Paragraph[] {
    return (
      this.#reversedQuoteIndex
        .get(id_to)
        ?.keys()
        .map((id_from) => this.#buffer.get(id_from))
        .filter((o) => o !== undefined)
        .toArray() ?? []
    );
  }
}

export type AnyYMapValue =
  | null
  | object
  | boolean
  | string
  | number
  | Uint8Array
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  | Y.AbstractType<any>;
