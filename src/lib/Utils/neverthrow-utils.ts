import {
  fromThrowable,
  fromAsyncThrowable,
  Result,
  ResultAsync,
  errAsync,
} from "neverthrow";

class UnknownExceptionError<T> extends Error {
  value: T;

  constructor(value: T) {
    super("An unknown exception occured.");
    this.name = this.constructor.name;
    this.value = value;
  }
}

export function handleError(value: unknown): Error {
  if (value instanceof Error) return value;
  if (typeof value === "symbol") return new UnknownExceptionError(value);
  return new UnknownExceptionError(value);
}

export function execThrowable<T>(fn: () => T): Result<T, Error> {
  return fromThrowable(fn, handleError)();
}

export function convertAsyncThrowable<T>(
  fn: () => Promise<T>,
): () => ResultAsync<T, Error> {
  return fromAsyncThrowable(fn, handleError);
}

export function execAsyncThrowable<T>(
  fn: () => Promise<T>,
): ResultAsync<T, Error> {
  return fromAsyncThrowable(fn, handleError)();
}

export function delay<T>(
  interval: number,
  fn: () => ResultAsync<T, Error>,
): ResultAsync<T, Error> {
  return ResultAsync.fromSafePromise(
    new Promise<ResultAsync<T, Error>>((resolve) => {
      setTimeout(() => {
        resolve(fn());
      }, interval);
    }),
  ).andThen((result) => result);
}

export function retry<T>(
  fn: (...args: readonly unknown[]) => ResultAsync<T, Error>,
  limit: number,
  interval: number = 0,
): ResultAsync<T, Error> {
  return fn().orElse((err) => {
    console.error(err.message);
    console.log("retrying...");
    if (limit !== 0) return delay(interval, () => retry(fn, limit - 1));
    return errAsync(err);
  });
}
