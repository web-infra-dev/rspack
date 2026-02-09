# rspack_paths

## Role
Path utilities and UTF‑8 path wrappers.

## Profiling relevance
- Path hashing and conversions show up indirectly (e.g., `Ustr::from`, path hashing).
- Hot in resolver and module graph operations.

## Perf opportunities
- Cache normalized path representations to avoid repeated conversions.
- Avoid `to_string_lossy` unless needed for diagnostics.
- Reduce hashing of path types in hot loops by caching hashes.

## Suggested experiments
- Measure path normalization overhead in large module graphs.
- Compare cached path hashing vs direct hashing.

## Code pointers
- `crates/rspack_paths/Cargo.toml`
- `crates/rspack_paths/src/lib.rs`
- `crates/rspack_paths/**`
