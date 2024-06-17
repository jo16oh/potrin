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
  let result = $state<T | undefined>(undefined);
  let unsubscribe: () => void = () => {};
  const hooks = new Map<symbol, () => void>();
  const preHooks = new Map<symbol, () => void>();

  function subscribe() {
    query().then((r) => {
      const tablenames = r.tablenames;
      result = r.result;
      execHooks(hooks);
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
  }

  execPreHooks(preHooks);
  subscribe();

  return {
    get result() {
      return result;
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
      return subscribe;
    },
  };
}
