/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Cache.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { createRequire } from 'node:module';
import type binding from '@rspack/binding';
import {
  AsyncParallelHook,
  AsyncSeriesBailHook,
  SyncHook,
} from '@rspack/lite-tapable';

import { makeWebpackError, makeWebpackErrorCallback } from './HookWebpackError';
import type { WebpackError } from './WebpackError';

const require = createRequire(import.meta.url);

export interface Etag {
  toString(): string;
}

export type CallbackCache<T> = (err?: WebpackError | null, result?: T) => void;

type GotHandler<T = any> = (
  result: T | null,
  callback: (error: Error | null) => void,
) => void;

const needCalls = (
  times: number,
  callback: () => void,
): ((error: Error | null) => void) => {
  let leftTimes = times;
  return (err) => {
    if (--leftTimes === 0) {
      return callback();
    }
    if (err && leftTimes > 0) {
      leftTimes = 0;
      return callback();
    }
  };
};

export class Cache {
  static STAGE_DISK = 10;
  static STAGE_MEMORY = -10;
  static STAGE_DEFAULT = 0;
  static STAGE_NETWORK = 20;

  #binding: binding.JsCache;

  hooks: {
    get: AsyncSeriesBailHook<[string, Etag | null, GotHandler[]], any>;
    store: AsyncParallelHook<[string, Etag | null, any]>;
    storeBuildDependencies: AsyncParallelHook<[Iterable<string>]>;
    beginIdle: SyncHook<[]>;
    endIdle: AsyncParallelHook<[]>;
    shutdown: AsyncParallelHook<[]>;
  };

  constructor() {
    this.hooks = {
      get: new AsyncSeriesBailHook(['identifier', 'etag', 'gotHandlers']),
      store: new AsyncParallelHook(['identifier', 'etag', 'data']),
      storeBuildDependencies: new AsyncParallelHook(['dependencies']),
      beginIdle: new SyncHook([]),
      endIdle: new AsyncParallelHook([]),
      shutdown: new AsyncParallelHook([]),
    };
    const { JsCache } = require('@rspack/binding') as typeof binding;
    this.#binding = new JsCache();
  }

  static __to_binding(cache: Cache): binding.JsCache {
    return cache.#binding;
  }

  /**
   * @param identifier the cache identifier
   * @param etag the etag
   * @param callback signals when the value is retrieved
   * @returns
   */
  get<T>(identifier: string, etag: Etag | null, callback: CallbackCache<T>) {
    const gotHandlers: GotHandler[] = [];

    this.hooks.get.callAsync(identifier, etag, gotHandlers, (err, res) => {
      if (err) {
        callback(makeWebpackError(err, 'Cache.hooks.get'));
        return;
      }
      let result = res;
      if (result === null) {
        result = undefined;
      }
      if (gotHandlers.length > 1) {
        const innerCallback = needCalls(gotHandlers.length, () =>
          callback(null, result),
        );
        for (const gotHandler of gotHandlers) {
          gotHandler(result, innerCallback);
        }
      } else if (gotHandlers.length === 1) {
        gotHandlers[0](result, () => callback(null, result));
      } else {
        callback(null, result);
      }
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
  ) {
    this.hooks.store.callAsync(
      identifier,
      etag,
      data,
      makeWebpackErrorCallback(callback, 'Cache.hooks.store'),
    );
  }

  /**
   * After this method has succeeded the cache can only be restored when build dependencies are
   * @param dependencies list of all build dependencies
   * @param callback signals when the dependencies are stored
   * @returns
   */
  storeBuildDependencies(
    dependencies: Iterable<string>,
    callback: CallbackCache<void>,
  ) {
    this.hooks.storeBuildDependencies.callAsync(
      dependencies,
      makeWebpackErrorCallback(callback, 'Cache.hooks.storeBuildDependencies'),
    );
  }

  beginIdle() {
    this.hooks.beginIdle.call();
    this.#binding.beginIdle();
  }

  /**
   * @param callback signals when the call finishes
   * @returns
   */
  endIdle(callback: CallbackCache<void>) {
    this.#binding.endIdle();
    this.hooks.endIdle.callAsync(
      makeWebpackErrorCallback(callback, 'Cache.hooks.endIdle'),
    );
  }

  /**
   * @param callback signals when the call finishes
   * @returns
   */
  shutdown(callback: CallbackCache<void>) {
    this.#binding.shutdown().then(() => {
      this.hooks.shutdown.callAsync(
        makeWebpackErrorCallback(callback, 'Cache.hooks.shutdown'),
      );
    });
  }
}

export default Cache;
