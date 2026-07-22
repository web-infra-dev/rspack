/**
 * Options for `builtin:cache-loader`.
 *
 * The builtin loader caches the result of the loaders that follow it.
 */
export interface CacheLoaderOptions {
  /**
   * Directory used for cache entries.
   * @default '<context>/node_modules/.cache/cache-loader'
   */
  cacheDirectory?: string;

  /** Identifier included in every cache key for manual invalidation. */
  cacheIdentifier?: string;
}
