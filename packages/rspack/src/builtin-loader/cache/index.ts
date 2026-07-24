/**
 * Options for `builtin:cache-loader`.
 *
 * The builtin loader intentionally has no options. It keeps the output of the
 * loaders that follow it in memory for the lifetime of the compiler.
 */
export type CacheLoaderOptions = Record<string, never>;
