// @ts-nocheck
/* eslint-disable */
import type { ThreadTree } from './../lib/Models/queries/getThreadTree.sql';

const isString = (value: unknown): value is string => typeof value === 'string';
const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
type ArrayCheckOption = 'all' | 'first';
const isArray = <T>(
  childCheckFn:
    | ((value: unknown) => value is T)
    | ((value: unknown) => boolean),
  checkOption: ArrayCheckOption = 'all'
) => (array: unknown): boolean =>
  Array.isArray(array) &&
  (checkOption === 'all'
    ? ((array) => {
        for (const val of array) {
          if (!childCheckFn(val)) return false
        }
        return true;
      })(array)
    : typeof array[0] === "undefined" || childCheckFn(array[0]));

export const isThreadTree = (arg_0: unknown): arg_0 is ThreadTree => isObject(arg_0) && 
  ('id' in arg_0 && (isString)(arg_0['id'])) && ('title' in arg_0 && (isString)(arg_0['title'])) && ('fractional_index' in arg_0 && (isString)(arg_0['fractional_index'])) && ('parent_thread' in arg_0 && (isThreadTree)(arg_0['parent_thread'])) && ('child_threads' in arg_0 && ((arg_1: unknown): boolean => isArray(isThreadTree)(arg_1))(arg_0['child_threads']));
