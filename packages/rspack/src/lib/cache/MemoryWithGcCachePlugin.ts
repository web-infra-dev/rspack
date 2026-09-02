/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/main/lib/cache/MemoryWithGcCachePlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from '../../Compiler';
import { Cache } from '../Cache';

type CacheEntry = {
  etag: string | null;
  data: unknown;
};

const getEtag = (etag: unknown): string | null =>
  typeof (etag as { toString?: unknown } | null)?.toString === 'function'
    ? String(etag)
    : null;

export default class MemoryWithGcCachePlugin {
  static PLUGIN_NAME = 'MemoryWithGcCachePlugin';

  private readonly maxGenerations: number;

  constructor({ maxGenerations }: { maxGenerations: number }) {
    this.maxGenerations = maxGenerations;
  }

  apply(compiler: Compiler) {
    const maxGenerations = this.maxGenerations;
    const cache = new Map<string, CacheEntry | null | undefined>();
    const oldCache = new Map<
      string,
      { entry: CacheEntry | null; until: number }
    >();
    let generation = 0;
    let cachePosition = 0;
    const logger = compiler.getInfrastructureLogger(
      MemoryWithGcCachePlugin.PLUGIN_NAME,
    );

    compiler.hooks.afterDone.tap(MemoryWithGcCachePlugin.PLUGIN_NAME, () => {
      generation++;
      let clearedEntries = 0;
      let lastClearedIdentifier: string | undefined;

      for (const [identifier, entry] of oldCache) {
        if (entry.until > generation) break;
        oldCache.delete(identifier);
        if (cache.get(identifier) === undefined) {
          cache.delete(identifier);
          clearedEntries++;
          lastClearedIdentifier = identifier;
        }
      }

      if (clearedEntries > 0 || oldCache.size > 0) {
        logger.log(
          `${cache.size - oldCache.size} active entries, ${
            oldCache.size
          } recently unused cached entries${
            clearedEntries > 0
              ? `, ${clearedEntries} old unused cache entries removed e. g. ${lastClearedIdentifier}`
              : ''
          }`,
        );
      }

      let count = (cache.size / maxGenerations) | 0;
      let skipped = cachePosition >= cache.size ? 0 : cachePosition;
      cachePosition = skipped + count;
      for (const [identifier, entry] of cache) {
        if (skipped !== 0) {
          skipped--;
          continue;
        }
        if (entry !== undefined) {
          cache.set(identifier, undefined);
          oldCache.delete(identifier);
          oldCache.set(identifier, {
            entry,
            until: generation + maxGenerations,
          });
          if (count-- === 0) break;
        }
      }
    });

    compiler.cache.hooks.store.tap(
      {
        name: MemoryWithGcCachePlugin.PLUGIN_NAME,
        stage: Cache.STAGE_MEMORY,
      },
      (identifier, etag, data) => {
        cache.set(identifier, { etag: getEtag(etag), data });
      },
    );
    compiler.cache.hooks.get.tap(
      {
        name: MemoryWithGcCachePlugin.PLUGIN_NAME,
        stage: Cache.STAGE_MEMORY,
      },
      (identifier, etag, gotHandlers) => {
        const dataEtag = getEtag(etag);
        const cacheEntry = cache.get(identifier);
        if (cacheEntry === null) return null;

        let known = cacheEntry !== undefined;
        if (cacheEntry !== undefined) {
          if (cacheEntry.etag === dataEtag) return cacheEntry.data;
        } else {
          const oldCacheEntry = oldCache.get(identifier);
          if (oldCacheEntry !== undefined) {
            const entry = oldCacheEntry.entry;
            if (entry === null) {
              oldCache.delete(identifier);
              cache.set(identifier, entry);
              return null;
            }
            known = true;
            if (entry.etag === dataEtag) {
              oldCache.delete(identifier);
              cache.set(identifier, entry);
              return entry.data;
            }
          }
        }

        gotHandlers.push((result, callback) => {
          if (result !== undefined) {
            cache.set(identifier, { etag: dataEtag, data: result });
          } else if (!known) {
            cache.set(identifier, null);
          }
          return callback(null);
        });
      },
    );
    compiler.cache.hooks.shutdown.tap(
      {
        name: MemoryWithGcCachePlugin.PLUGIN_NAME,
        stage: Cache.STAGE_MEMORY,
      },
      () => {
        cache.clear();
        oldCache.clear();
      },
    );
  }
}
