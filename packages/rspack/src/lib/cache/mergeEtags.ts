/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/cache/mergeEtags.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Etag } from '../Cache';

class MergedEtag {
  a: Etag;
  b: Etag;

  /**
   * @param a first
   * @param b second
   */
  constructor(a: Etag, b: Etag) {
    this.a = a;
    this.b = b;
  }

  toString() {
    return `${this.a.toString()}|${this.b.toString()}`;
  }
}

const dualObjectMap = new WeakMap();
const objectStringMap = new WeakMap();

/**
 * @param first first
 * @param second second
 * @returns result
 */
export const mergeEtags = (first: Etag, second: Etag): Etag => {
  let a = first;
  let b = second;

  if (typeof a === 'string') {
    if (typeof b === 'string') {
      return `${a}|${b}`;
    }
    const temp = b;
    b = a;
    a = temp;
  } else {
    if (typeof b !== 'string') {
      // both a and b are objects
      let map = dualObjectMap.get(a);
      if (map === undefined) {
        dualObjectMap.set(a, (map = new WeakMap()));
      }
      const mergedEtag = map.get(b);
      if (mergedEtag === undefined) {
        const newMergedEtag = new MergedEtag(a, b);
        map.set(b, newMergedEtag);
        return newMergedEtag;
      }
      return mergedEtag;
    }
  }
  // a is object, b is string
  let map = objectStringMap.get(a);
  if (map === undefined) {
    objectStringMap.set(a, (map = new Map()));
  }
  const mergedEtag = map.get(b);
  if (mergedEtag === undefined) {
    const newMergedEtag = new MergedEtag(a, b);
    map.set(b, newMergedEtag);
    return newMergedEtag;
  }
  return mergedEtag;
};

export default mergeEtags;
