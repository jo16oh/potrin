// @ts-nocheck
/* eslint-disable */
import type { Card } from './../lib/Models/card';
import type { ThreadTree } from './../lib/Models/queries/getThreadTree.sql';
import type { Thread } from './../lib/Models/thread';

const isString = (value: unknown): value is string => typeof value === 'string';
const isNull = (value: unknown): value is null => value === null;
const isDate = (value: unknown): value is Date =>
  value instanceof Date || Object.prototype.toString.call(value) === '[Object Date]'
const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
const isUnion = (unionChecks: ((value: unknown) => boolean)[]) =>
  (value: unknown): boolean =>
    unionChecks.reduce((s: boolean, isT) => s || isT(value), false)
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

export const isCard = (arg_0: unknown): arg_0 is Card => isObject(arg_0) && 
  ('id' in arg_0 && (isString)(arg_0['id'])) && ('thread' in arg_0 && (isString)(arg_0['thread'])) && ('fractional_index' in arg_0 && (isString)(arg_0['fractional_index'])) && ('content' in arg_0 && ((arg_1: unknown): boolean => isUnion([isNull, isString])(arg_1))(arg_0['content'])) && ('created_at' in arg_0 && (isDate)(arg_0['created_at'])) && ('updated_at' in arg_0 && (isDate)(arg_0['updated_at'])) && ('deleted' in arg_0 && ((arg_1: unknown): boolean => isUnion([(arg_2: unknown): boolean => arg_2 === false, (arg_2: unknown): boolean => arg_2 === true])(arg_1))(arg_0['deleted']));

export const isThreadTree = (arg_0: unknown): arg_0 is ThreadTree => isObject(arg_0) && 
  ('id' in arg_0 && (isString)(arg_0['id'])) && ('title' in arg_0 && (isString)(arg_0['title'])) && ('fractional_index' in arg_0 && (isString)(arg_0['fractional_index'])) && ('parent_thread' in arg_0 && (isThreadTree)(arg_0['parent_thread'])) && ('cards' in arg_0 && ((arg_1: unknown): boolean => isArray(isCard)(arg_1))(arg_0['cards'])) && ('child_threads' in arg_0 && ((arg_1: unknown): boolean => isArray(isThreadTree)(arg_1))(arg_0['child_threads']));

export const isThread = (arg_0: unknown): arg_0 is Thread => isObject(arg_0) && 
  ('id' in arg_0 && (isString)(arg_0['id'])) && ('fractional_index' in arg_0 && (isString)(arg_0['fractional_index'])) && ('created_at' in arg_0 && (isDate)(arg_0['created_at'])) && ('updated_at' in arg_0 && (isDate)(arg_0['updated_at'])) && ('deleted' in arg_0 && ((arg_1: unknown): boolean => isUnion([(arg_2: unknown): boolean => arg_2 === false, (arg_2: unknown): boolean => arg_2 === true])(arg_1))(arg_0['deleted'])) && ('parent_thread' in arg_0 && ((arg_1: unknown): boolean => isUnion([isNull, isString])(arg_1))(arg_0['parent_thread'])) && ('title' in arg_0 && ((arg_1: unknown): boolean => isUnion([isNull, isString])(arg_1))(arg_0['title']));

