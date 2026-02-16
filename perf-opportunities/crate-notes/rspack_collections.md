# rspack_collections

## Role
Custom collection types used in hot paths (sets, maps, identifiers).

## Profiling relevance
- Indirectly visible via identifier maps and hash insertions.
- Hot when module graph and resolver perform many lookups.

## Perf opportunities
- Use cache‑friendly layouts for hot lookups.
- Avoid repeated hashing by caching keys where possible.
- Prefer specialized maps for identifier-heavy workloads.

## Key functions/structs to inspect
- `Identifier::precomputed_hash` (identifier.rs).
- `CustomConverter::serialize` / `deserialize` for Identifier (identifier.rs).
- `Ukey` helpers in `ukey.rs`.

## Suggested experiments
- Benchmark custom collections with realistic module graph sizes.
- Compare `FxHash`/custom hashers for identifier-heavy workloads.

## Code pointers
- `crates/rspack_collections/Cargo.toml`
- `crates/rspack_collections/src/lib.rs`
- `crates/rspack_collections/src/identifier.rs`
- `crates/rspack_collections/src/ukey.rs`
- `crates/rspack_collections/**`
