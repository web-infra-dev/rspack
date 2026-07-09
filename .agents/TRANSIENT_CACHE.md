# Transient cache

Read this before adding cache state to a compilation.

`transient_cache` stores data that is valid only within one compilation
lifecycle.

## Difference from artifacts

- An artifact can be recovered across compilations and may participate in
  incremental or persistent cache flows.
- `transient_cache` is re-initialized for each compilation and must not affect or
  depend on future compilation state.

## Use transient cache when

- The cached value only accelerates repeated work inside the current
  compilation.
- Reusing the value in a later compilation would risk stale state.
- The cache is derived from mutable compilation state that is not part of
  incremental recovery.

## Do not use transient cache when

- The data needs to survive rebuilds.
- The data is required to restore incremental state.
- The data should participate in persistent cache behavior.

Use an artifact instead when data must be recoverable across compilations.
