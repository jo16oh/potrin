// referenced https://github.com/conorbergin/svelte-electric-app/blob/main/src/lib/svelteLiveQuery.ts

import { QualifiedTablename, hasIntersection } from "electric-sql/util";
import { type Notifier } from "electric-sql/notifiers";

export interface LiveResultContext<T> {
  (): Promise<LiveResult<T>>;
  sourceQuery?: Record<string, any> | undefined;
}

export class LiveResult<T> {
  constructor(
    public result: T,
    public tablenames: QualifiedTablename[],
  ) {}
}

export class QueryStore<T> {
  #query = $state<LiveResultContext<T>>();
  get query() {
    return this.#query!;
  }
  set query(query: LiveResultContext<T>) {
    this.#query = query;
  }

  constructor(query: LiveResultContext<T>) {
    this.#query = query;
  }
}

function execPreHooks(map: Map<symbol, () => void>) {
  map.forEach((fn) => {
    try {
      fn();
    } catch (err) {
      console.error(err);
    }
  });
}

function execHooks(map: Map<symbol, () => void>) {
  map.forEach((fn) => {
    try {
      fn();
    } catch (err) {
      console.error(err);
    }
  });
}

export function createLiveQuery<T>(
  notifier: Notifier,
  query: LiveResultContext<T>,
) {
  let result = $state<T | null>(null);
  let unsubscribe: () => void = () => {};
  const hooks = new Map<symbol, () => void>();
  const preHooks = new Map<symbol, () => void>();

  query().then((r) => {
    const tablenames = r.tablenames;
    result = r.result;
    unsubscribe = notifier.subscribeToDataChanges((notification) => {
      const changedTablenames = notifier.alias(notification);
      if (hasIntersection(tablenames, changedTablenames)) {
        execPreHooks(preHooks);
        query()
          .then((r) => {
            result = r.result;
          })
          .then(() => execHooks(hooks));
      }
    });
  });

  return {
    get result() {
      return result;
    },
    set result(r) {
      result = r;
    },
    addHook(fn: () => void): () => void {
      const key = Symbol("key");
      hooks.set(key, fn);
      return () => {
        hooks.delete(key);
      };
    },
    addPreHook(fn: () => void): () => void {
      const key = Symbol("key");
      preHooks.set(key, fn);
      return () => {
        preHooks.delete(key);
      };
    },
    unsubscribe: () => {
      unsubscribe();
    },
  };
}
