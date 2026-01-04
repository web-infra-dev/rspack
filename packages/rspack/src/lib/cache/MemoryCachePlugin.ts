import type { Compiler } from '../../Compiler';
import { Cache } from '../Cache';

export default class MemoryCachePlugin {
  static PLUGIN_NAME = 'MemoryCachePlugin';
  apply(compiler: Compiler) {
    const cache: Map<
      string,
      {
        etag: string | null;
        data: unknown;
      } | null
    > = new Map();
    compiler.cache.hooks.store.tap(
      { name: MemoryCachePlugin.PLUGIN_NAME, stage: Cache.STAGE_MEMORY },
      (identifier, etag, data) => {
        const dataEtag =
          typeof etag?.toString === 'function'
            ? etag.toString()
            : (etag as null);
        cache.set(identifier, { etag: dataEtag, data });
      },
    );
    compiler.cache.hooks.get.tap(
      { name: MemoryCachePlugin.PLUGIN_NAME, stage: Cache.STAGE_MEMORY },
      (identifier, etag, gotHandlers) => {
        const cacheEntry = cache.get(identifier);
        const dataEtag =
          typeof etag?.toString === 'function'
            ? etag.toString()
            : (etag as null);
        if (cacheEntry === null) {
          return null;
        }
        if (cacheEntry !== undefined) {
          return cacheEntry.etag === dataEtag ? cacheEntry.data : null;
        }
        gotHandlers.push((result, callback) => {
          if (result === undefined) {
            cache.set(identifier, null);
          } else {
            cache.set(identifier, { etag: dataEtag, data: result });
          }
          return callback(null);
        });
      },
    );
    compiler.cache.hooks.shutdown.tap(
      { name: MemoryCachePlugin.PLUGIN_NAME, stage: Cache.STAGE_MEMORY },
      () => {
        cache.clear();
      },
    );
  }
}
