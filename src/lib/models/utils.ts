import type { Links } from "../../generated/tauri-commands";

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
        const backlinks = new Set<string>();
        backlinks.add(id_from);
        this.#reversedLinkIndex.set(id_to, backlinks);
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
