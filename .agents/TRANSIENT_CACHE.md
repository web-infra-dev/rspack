# Transient Cache

`transient_cache` stores cache data that is **only valid within a single compilation lifecycle**.

Difference from `artifact`:

- `artifact` is designed to be **reused across compilations** by Incremental recovery within the
  same compiler lifecycle. Its logical ownership and recovery path do not depend on Cache or the
  selected Cache backend. Historical physical co-location exceptions are documented in
  [Cache and Incremental Compilation](./CACHE_AND_INCREMENTAL.md).
- `transient_cache` is designed to **not be reused across compilations**. It is re-initialized each compilation, and should only accelerate the current build. It must not be part of incremental recovery or persistent cache flows.

Difference from Cache:

- Cache owns fine-grained entries whose lifetime is managed by a memory or filesystem backend.
- `transient_cache` is compilation-local and must not be stored in either Cache backend.

See [Cache and Incremental Compilation](./CACHE_AND_INCREMENTAL.md) for the complete ownership
model.

Use `transient_cache` when the cache must not affect or depend on future compilation state (avoid cross-compilation contamination or when strict isolation is required).
