# Transient Cache

`transient_cache` stores cache data that is **only valid within a single compilation lifecycle**.

Difference from `artifact`:
- `artifact` is designed to be **reused across compilations**, and participates in incremental recovery and persistent caching.
- `transient_cache` is designed to **not be reused across compilations**. It is re-initialized each compilation, and should only accelerate the current build. It must not be part of incremental recovery or persistent cache flows.

Use `transient_cache` when the cache must not affect or depend on future compilation state (avoid cross-compilation contamination or when strict isolation is required).
