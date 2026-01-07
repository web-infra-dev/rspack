/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/CacheFacade.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Cache, CallbackCache, Etag } from './Cache';
import type {
  HashableObject,
  HashConstructor,
} from './cache/getLazyHashedEtag';
import { getter as getLazyHashedEtag } from './cache/getLazyHashedEtag.js';
import { mergeEtags } from './cache/mergeEtags.js';
import type WebpackError from './WebpackError';

type CallbackNormalErrorCache<T> = (
  err?: WebpackError | null,
  result?: T,
) => void;

abstract class BaseCache {
  abstract get<T>(callback: CallbackCache<T>): void;
  abstract getPromise<T>(): Promise<T | undefined>;
  abstract store<T>(data: T, callback: CallbackCache<void>): void;
  abstract storePromise<T>(data: T): Promise<void>;
}

export class ItemCacheFacade implements BaseCache {
  _cache: Cache;
  _name: string;
  _etag: Etag | null;

  /**
   * @param cache the root cache
   * @param name the child cache item name
   * @param etag the etag
   * @returns
   */
  constructor(cache: Cache, name: string, etag: Etag | null) {
    this._cache = cache;
    this._name = name;
    this._etag = etag;
  }

  /**
   * @param callback signals when the value is retrieved
   * @returns
   */
  get<T>(callback: CallbackCache<T>): void {
    this._cache.get(this._name, this._etag, callback);
  }

  /**
   * @returns promise with the data
   */
  getPromise<T>(): Promise<T | undefined> {
    return new Promise((resolve, reject) => {
      this._cache.get<T>(this._name, this._etag, (err, data) => {
        if (err) {
          reject(err);
        } else {
          resolve(data);
        }
      });
    });
  }

  /**
   * @param data the value to store
   * @param callback signals when the value is stored
   * @returns
   */
  store<T>(data: T, callback: CallbackCache<void>): void {
    this._cache.store(this._name, this._etag, data, callback);
  }

  /**
   * @param data the value to store
   * @returns promise signals when the value is stored
   */
  storePromise<T>(data: T): Promise<void> {
    return new Promise((resolve, reject) => {
      this._cache.store(this._name, this._etag, data, (err) => {
        if (err) {
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * @param computer function to compute the value if not cached
   * @param callback signals when the value is retrieved
   * @returns
   */
  provide<T>(
    computer: (callback: CallbackNormalErrorCache<T>) => void,
    callback: CallbackNormalErrorCache<T>,
  ) {
    this.get((err, cacheEntry) => {
      if (err) return callback(err);
      if (cacheEntry !== undefined) return cacheEntry;
      computer((err, result) => {
        if (err) return callback(err);
        this.store(result, (err) => {
          if (err) return callback(err);
          callback(null, result);
        });
      });
    });
  }

  /**
   * @param computer function to compute the value if not cached
   * @returns promise with the data
   */
  async providePromise<T>(computer: () => Promise<T> | T): Promise<T> {
    const cacheEntry = await this.getPromise<T>();
    if (cacheEntry !== undefined) return cacheEntry;
    const result = await computer();
    await this.storePromise(result);
    return result;
  }
}

export class CacheFacade {
  _name: string;
  _cache: Cache;
  _hashFunction: string | HashConstructor;

  /**
   * @param cache the root cache
   * @param name the child cache name
   * @param hashFunction the hash function to use
   */
  constructor(
    cache: Cache,
    name: string,
    hashFunction: string | HashConstructor,
  ) {
    this._cache = cache;
    this._name = name;
    this._hashFunction = hashFunction;
  }

  /**
   * @param name the child cache name#
   * @returns child cache
   */
  getChildCache(name: string): CacheFacade {
    return new CacheFacade(
      this._cache,
      `${this._name}|${name}`,
      this._hashFunction,
    );
  }

  /**
   * @param identifier the cache identifier
   * @param  etag the etag
   * @returns item cache
   */
  getItemCache(identifier: string, etag: Etag | null): ItemCacheFacade {
    return new ItemCacheFacade(
      this._cache,
      `${this._name}|${identifier}`,
      etag,
    );
  }

  /**
   * @param obj an hashable object
   * @returns an etag that is lazy hashed
   */
  getLazyHashedEtag(obj: HashableObject): Etag {
    return getLazyHashedEtag(obj, this._hashFunction);
  }

  /**
   * @param a an etag
   * @param b another etag
   * @returns an etag that represents both
   */
  mergeEtags(a: Etag, b: Etag): Etag {
    return mergeEtags(a, b);
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param callback signals when the value is retrieved
   * @returns
   */
  get<T>(
    identifier: string,
    etag: Etag | null,
    callback: CallbackCache<T>,
  ): void {
    this._cache.get(`${this._name}|${identifier}`, etag, callback);
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @returns promise with the data
   */
  getPromise<T>(identifier: string, etag: Etag | null): Promise<T | undefined> {
    return new Promise((resolve, reject) => {
      this._cache.get<T>(`${this._name}|${identifier}`, etag, (err, data) => {
        if (err) {
          reject(err);
        } else {
          resolve(data);
        }
      });
    });
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param data the value to store
   * @param callback signals when the value is stored
   * @returns
   */
  store<T>(
    identifier: string,
    etag: Etag | null,
    data: T,
    callback: CallbackCache<void>,
  ): void {
    this._cache.store(`${this._name}|${identifier}`, etag, data, callback);
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param data the value to store
   * @returns promise signals when the value is stored
   */
  storePromise<T>(
    identifier: string,
    etag: Etag | null,
    data: T,
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      this._cache.store<T>(`${this._name}|${identifier}`, etag, data, (err) => {
        if (err) {
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param computer function to compute the value if not cached
   * @param callback signals when the value is retrieved
   * @returns
   */
  provide<T>(
    identifier: string,
    etag: Etag | null,
    computer: (callback: CallbackNormalErrorCache<T>) => void,
    callback: CallbackNormalErrorCache<T>,
  ): void {
    this.get<T>(identifier, etag, (err, cacheEntry) => {
      if (err) return callback(err);
      if (cacheEntry !== undefined) return cacheEntry;
      computer((err, result) => {
        if (err) return callback(err);
        this.store(identifier, etag, result, (err) => {
          if (err) return callback(err);
          callback(null, result);
        });
      });
    });
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param computer function to compute the value if not cached
   * @returns promise with the data
   */
  async providePromise<T>(
    identifier: string,
    etag: Etag | null,
    computer: () => Promise<T> | T,
  ) {
    const cacheEntry = await this.getPromise(identifier, etag);
    if (cacheEntry !== undefined) return cacheEntry;
    const result = await computer();
    await this.storePromise(identifier, etag, result);
    return result;
  }
}

export default CacheFacade;
