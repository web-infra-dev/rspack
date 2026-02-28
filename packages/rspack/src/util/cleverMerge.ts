/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/cleverMerge.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
type Obj = Record<PropertyKey, any>;

type Info = Map<any, ObjectParsedPropertyEntry>;

const DYNAMIC_INFO = Symbol('cleverMerge dynamic info');
type FunctionWithDynamicInfo = ((...args: any[]) => any) & {
  [DYNAMIC_INFO]?: [FunctionWithDynamicInfo, Obj];
};

type DynamicInfo = {
  byProperty: string;
  fn: FunctionWithDynamicInfo;
};

type ParsedObject = {
  /**
   * static properties (key is property name)
   */
  static: Map<string, ObjectParsedPropertyEntry>;
  /**
   * dynamic part
   */
  dynamic: DynamicInfo | undefined;
};

type ObjectParsedPropertyEntry = {
  /**
   * base value
   */
  base: any;
  /**
   * the name of the selector property
   */
  byProperty: string | undefined;
  /**
   * value depending on selector property, merged with base
   */
  byValues: Map<string, any>;
};

const mergeCache = new WeakMap<Obj, WeakMap<Obj, Obj>>();

export const DELETE = Symbol('DELETE');

/**
 * Merges two given objects and caches the result to avoid computation if same objects passed as arguments again.
 * @example
 * // performs cleverMerge(first, second), stores the result in WeakMap and returns result
 * cachedCleverMerge({a: 1}, {a: 2})
 * {a: 2}
 *  // when same arguments passed, gets the result from WeakMap and returns it.
 * cachedCleverMerge({a: 1}, {a: 2})
 * {a: 2}
 * @param first first object
 * @param second second object
 * @returns  merged object of first and second object
 */
export const cachedCleverMerge = <First, Second>(
  first: First,
  second: Second,
): First | Second | (First & Second) => {
  if (second === undefined) return first;
  if (first === undefined) return second;
  if (typeof second !== 'object' || second === null) return second;
  if (typeof first !== 'object' || first === null) return first;

  let innerCache = mergeCache.get(first);
  if (innerCache === undefined) {
    innerCache = new WeakMap();
    mergeCache.set(first, innerCache);
  }
  const prevMerge = innerCache.get(second);

  if (prevMerge !== undefined) return prevMerge;
  const newMerge = _cleverMerge(first, second, true);
  innerCache.set(second, newMerge);
  return newMerge;
};

const parseCache = new WeakMap<Obj, ParsedObject>();

/**
 * @param obj the object
 * @returns parsed object
 */
const cachedParseObject = (obj: Obj) => {
  const entry = parseCache.get(obj);
  if (entry !== undefined) return entry;
  const result = parseObject(obj);
  parseCache.set(obj, result);
  return result;
};

/**
 * @param {object} obj the object
 * @returns {ParsedObject} parsed object
 */
const parseObject = (obj: Obj): ParsedObject => {
  const info: Info = new Map();
  let dynamicInfo: DynamicInfo | undefined;
  const getInfo = (p: any) => {
    const entry = info.get(p);
    if (entry !== undefined) return entry;
    const newEntry = {
      base: undefined,
      byProperty: undefined,
      byValues: new Map(),
    };
    info.set(p, newEntry);
    return newEntry;
  };
  for (const key of Object.keys(obj)) {
    if (key.startsWith('by')) {
      const byProperty = key;
      const byObj = obj[byProperty];
      if (typeof byObj === 'object') {
        for (const byValue of Object.keys(byObj)) {
          const obj = byObj[byValue];
          for (const key of Object.keys(obj)) {
            const entry = getInfo(key);
            if (entry.byProperty === undefined) {
              entry.byProperty = byProperty;
            } else if (entry.byProperty !== byProperty) {
              throw new Error(
                `${byProperty} and ${entry.byProperty} for a single property is not supported`,
              );
            }
            entry.byValues.set(byValue, obj[key]);
            if (byValue === 'default') {
              for (const otherByValue of Object.keys(byObj)) {
                if (!entry.byValues.has(otherByValue))
                  entry.byValues.set(otherByValue, undefined);
              }
            }
          }
        }
      } else if (typeof byObj === 'function') {
        if (dynamicInfo === undefined) {
          dynamicInfo = {
            byProperty: key,
            fn: byObj,
          };
        } else {
          throw new Error(
            `${key} and ${dynamicInfo.byProperty} when both are functions is not supported`,
          );
        }
      } else {
        const entry = getInfo(key);
        entry.base = obj[key];
      }
    } else {
      const entry = getInfo(key);
      entry.base = obj[key];
    }
  }
  return {
    static: info,
    dynamic: dynamicInfo,
  };
};

/**
 * @param info static properties (key is property name)
 * @param dynamicInfo dynamic part
 * @returns the object
 */
const serializeObject = (info: Info, dynamicInfo: DynamicInfo | undefined) => {
  const obj: Obj = {};
  // Setup byProperty structure
  for (const entry of info.values()) {
    if (entry.byProperty !== undefined) {
      const byObj = (obj[entry.byProperty] = obj[entry.byProperty] || {});
      for (const byValue of entry.byValues.keys()) {
        byObj[byValue] = byObj[byValue] || {};
      }
    }
  }
  for (const [key, entry] of info) {
    if (entry.base !== undefined) {
      obj[key] = entry.base;
    }
    // Fill byProperty structure
    if (entry.byProperty !== undefined) {
      const byObj = (obj[entry.byProperty] = obj[entry.byProperty] || {});
      for (const byValue of Object.keys(byObj)) {
        const value = getFromByValues(entry.byValues, byValue);
        if (value !== undefined) byObj[byValue][key] = value;
      }
    }
  }
  if (dynamicInfo !== undefined) {
    obj[dynamicInfo.byProperty] = dynamicInfo.fn;
  }
  return obj;
};

const VALUE_TYPE_UNDEFINED = 0;
const VALUE_TYPE_ATOM = 1;
const VALUE_TYPE_ARRAY_EXTEND = 2;
const VALUE_TYPE_OBJECT = 3;
const VALUE_TYPE_DELETE = 4;

/**
 * @param value a single value
 * @returns {VALUE_TYPE_UNDEFINED | VALUE_TYPE_ATOM | VALUE_TYPE_ARRAY_EXTEND | VALUE_TYPE_OBJECT | VALUE_TYPE_DELETE} value type
 */
const getValueType = (value: any) => {
  if (value === undefined) {
    return VALUE_TYPE_UNDEFINED;
  }
  if (value === DELETE) {
    return VALUE_TYPE_DELETE;
  }
  if (Array.isArray(value)) {
    if (value.lastIndexOf('...') !== -1) return VALUE_TYPE_ARRAY_EXTEND;
    return VALUE_TYPE_ATOM;
  }
  if (
    typeof value === 'object' &&
    value !== null &&
    (!value.constructor || value.constructor === Object)
  ) {
    return VALUE_TYPE_OBJECT;
  }
  return VALUE_TYPE_ATOM;
};

/**
 * Merges two objects. Objects are deeply clever merged.
 * Arrays might reference the old value with "...".
 * Non-object values take preference over object values.
 * @param first first object
 * @param second second object
 * @returns merged object of first and second object
 */
export const cleverMerge = <First, Second>(
  first: First,
  second: Second,
): First | Second | (First & Second) => {
  if (second === undefined) return first;
  if (first === undefined) return second;
  if (typeof second !== 'object' || second === null) return second;
  if (typeof first !== 'object' || first === null) return first;

  return _cleverMerge(first, second, false);
};

/**
 * Merges two objects. Objects are deeply clever merged.
 * @param first first object
 * @param second second object
 * @param internalCaching should parsing of objects and nested merges be cached
 * @returns merged object of first and second object
 */
const _cleverMerge = <First extends Obj, Second extends Obj>(
  first: First,
  second: Second,
  internalCaching = false,
): First & Second => {
  const firstObject = internalCaching
    ? cachedParseObject(first)
    : parseObject(first);
  const { static: firstInfo, dynamic: firstDynamicInfo } = firstObject;

  // If the first argument has a dynamic part we modify the dynamic part to merge the second argument
  let secondObj = second;
  if (firstDynamicInfo !== undefined) {
    let { byProperty, fn } = firstDynamicInfo;
    const fnInfo = fn[DYNAMIC_INFO];
    if (fnInfo) {
      secondObj = internalCaching
        ? cachedCleverMerge(fnInfo[1], second)
        : cleverMerge(fnInfo[1], second);
      fn = fnInfo[0];
    }

    const newFn: FunctionWithDynamicInfo = (...args: any[]) => {
      const fnResult = fn(...args);
      return internalCaching
        ? cachedCleverMerge(fnResult, secondObj)
        : cleverMerge(fnResult, secondObj);
    };

    newFn[DYNAMIC_INFO] = [fn, secondObj];
    return serializeObject(firstObject.static, { byProperty, fn: newFn });
  }

  // If the first part is static only, we merge the static parts and keep the dynamic part of the second argument
  const secondObject = internalCaching
    ? cachedParseObject(second)
    : parseObject(second);
  const { static: secondInfo, dynamic: secondDynamicInfo } = secondObject;
  const resultInfo = new Map<string, ObjectParsedPropertyEntry>();
  for (const [key, firstEntry] of firstInfo) {
    const secondEntry = secondInfo.get(key);
    const entry =
      secondEntry !== undefined
        ? mergeEntries(firstEntry, secondEntry, internalCaching)
        : firstEntry;
    resultInfo.set(key, entry);
  }
  for (const [key, secondEntry] of secondInfo) {
    if (!firstInfo.has(key)) {
      resultInfo.set(key, secondEntry);
    }
  }
  return serializeObject(resultInfo, secondDynamicInfo);
};

/**
 * @param firstEntry a
 * @param secondEntry b
 * @param internalCaching should parsing of objects and nested merges be cached
 * @returns new entry
 */
const mergeEntries = (
  firstEntry: ObjectParsedPropertyEntry,
  secondEntry: ObjectParsedPropertyEntry,
  internalCaching: boolean,
): ObjectParsedPropertyEntry => {
  switch (getValueType(secondEntry.base)) {
    case VALUE_TYPE_ATOM:
    case VALUE_TYPE_DELETE:
      // No need to consider firstEntry at all
      // second value override everything
      // = second.base + second.byProperty
      return secondEntry;
    case VALUE_TYPE_UNDEFINED: {
      if (!firstEntry.byProperty) {
        // = first.base + second.byProperty
        return {
          base: firstEntry.base,
          byProperty: secondEntry.byProperty,
          byValues: secondEntry.byValues,
        };
      }
      if (firstEntry.byProperty !== secondEntry.byProperty) {
        throw new Error(
          `${firstEntry.byProperty} and ${secondEntry.byProperty} for a single property is not supported`,
        );
      }
      // = first.base + (first.byProperty + second.byProperty)
      // need to merge first and second byValues
      const newByValues = new Map(firstEntry.byValues);
      for (const [key, value] of secondEntry.byValues) {
        const firstValue = getFromByValues(firstEntry.byValues, key);
        newByValues.set(
          key,
          mergeSingleValue(firstValue, value, internalCaching),
        );
      }
      return {
        base: firstEntry.base,
        byProperty: firstEntry.byProperty,
        byValues: newByValues,
      };
    }
    default: {
      if (!firstEntry.byProperty) {
        // The simple case
        // = (first.base + second.base) + second.byProperty
        return {
          base: mergeSingleValue(
            firstEntry.base,
            secondEntry.base,
            internalCaching,
          ),
          byProperty: secondEntry.byProperty,
          byValues: secondEntry.byValues,
        };
      }
      let newBase: ObjectParsedPropertyEntry['base'];
      const intermediateByValues = new Map(firstEntry.byValues);
      for (const [key, value] of intermediateByValues) {
        intermediateByValues.set(
          key,
          mergeSingleValue(value, secondEntry.base, internalCaching),
        );
      }
      if (
        Array.from(firstEntry.byValues.values()).every((value) => {
          const type = getValueType(value);
          return type === VALUE_TYPE_ATOM || type === VALUE_TYPE_DELETE;
        })
      ) {
        // = (first.base + second.base) + ((first.byProperty + second.base) + second.byProperty)
        newBase = mergeSingleValue(
          firstEntry.base,
          secondEntry.base,
          internalCaching,
        );
      } else {
        // = first.base + ((first.byProperty (+default) + second.base) + second.byProperty)
        newBase = firstEntry.base;
        if (!intermediateByValues.has('default'))
          intermediateByValues.set('default', secondEntry.base);
      }
      if (!secondEntry.byProperty) {
        // = first.base + (first.byProperty + second.base)
        return {
          base: newBase,
          byProperty: firstEntry.byProperty,
          byValues: intermediateByValues,
        };
      }
      if (firstEntry.byProperty !== secondEntry.byProperty) {
        throw new Error(
          `${firstEntry.byProperty} and ${secondEntry.byProperty} for a single property is not supported`,
        );
      }
      const newByValues = new Map(intermediateByValues);
      for (const [key, value] of secondEntry.byValues) {
        const firstValue = getFromByValues(intermediateByValues, key);
        newByValues.set(
          key,
          mergeSingleValue(firstValue, value, internalCaching),
        );
      }
      return {
        base: newBase,
        byProperty: firstEntry.byProperty,
        byValues: newByValues,
      };
    }
  }
};

/**
 * @param byValues all values
 * @param key value of the selector
 * @returns value
 */
const getFromByValues = (byValues: Obj, key: string) => {
  if (key !== 'default' && byValues.has(key)) {
    return byValues.get(key);
  }
  return byValues.get('default');
};

/**
 * @param a value
 * @param b value
 * @param internalCaching should parsing of objects and nested merges be cached
 * @returns value
 */
const mergeSingleValue = (a: any, b: any, internalCaching: boolean) => {
  const bType = getValueType(b);
  const aType = getValueType(a);
  switch (bType) {
    case VALUE_TYPE_DELETE:
    case VALUE_TYPE_ATOM:
      return b;
    case VALUE_TYPE_OBJECT: {
      return aType !== VALUE_TYPE_OBJECT
        ? b
        : internalCaching
          ? cachedCleverMerge(a, b)
          : cleverMerge(a, b);
    }
    case VALUE_TYPE_UNDEFINED:
      return a;
    case VALUE_TYPE_ARRAY_EXTEND:
      switch (
        aType !== VALUE_TYPE_ATOM
          ? aType
          : Array.isArray(a)
            ? VALUE_TYPE_ARRAY_EXTEND
            : VALUE_TYPE_OBJECT
      ) {
        case VALUE_TYPE_UNDEFINED:
          return b;
        case VALUE_TYPE_DELETE:
          return b.filter((item: string) => item !== '...');
        case VALUE_TYPE_ARRAY_EXTEND: {
          const newArray = [];
          for (const item of b) {
            if (item === '...') {
              for (const item of a) {
                newArray.push(item);
              }
            } else {
              newArray.push(item);
            }
          }
          return newArray;
        }
        case VALUE_TYPE_OBJECT:
          return b.map((item: string) => (item === '...' ? a : item));
        default:
          throw new Error('Not implemented');
      }
    default:
      throw new Error('Not implemented');
  }
};
