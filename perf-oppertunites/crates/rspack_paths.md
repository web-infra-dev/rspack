# rspack_paths

## Role
Path utilities and UTF‑8 path wrappers.

## Perf opportunities
- Cache normalized path representations to avoid repeated conversions.
- Avoid `to_string_lossy` unless needed for diagnostics.
- Reduce hashing of path types in hot loops by caching hashes.

## Code pointers
- `crates/rspack_paths/**`
