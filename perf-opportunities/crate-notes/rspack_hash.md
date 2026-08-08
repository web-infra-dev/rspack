# rspack_hash

## Role
Hashing utilities used for modules/chunks/assets.

## Profiling relevance
- Hashing costs show up in chunk/module hash passes.
- Costs scale with module count and asset sizes.

## Perf opportunities
- Cache computed hashes and reuse across passes.
- Avoid hashing when content hash unchanged (use dirty flags).
- Prefer incremental hashing where possible.
- Single-file crate: concentrate profiling on `src/lib.rs` hashing routines.

## Key functions/structs to inspect
- `RspackHash::new` / `RspackHash::with_salt` (lib.rs).
- `RspackHash::digest` (lib.rs).
- `Hasher::write` implementation (lib.rs).

## Suggested experiments
- Measure hash time with and without caching on large builds.
- Compare incremental vs full hash strategies.

## Code pointers
- `crates/rspack_hash/Cargo.toml`
- `crates/rspack_hash/src/lib.rs`
- `crates/rspack_hash/**`
