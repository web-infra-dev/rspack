# rspack_plugin_copy

## Role
Copy assets or files to output directory.

## Profiling relevance
- Not visible in react-10k; hot when large copy patterns are configured.
- Costs scale with file count and IO throughput.

## Perf opportunities
- Batch file copy operations to reduce IO overhead.
- Avoid hashing or reading files when unchanged.
- Skip copy pass when configuration has no patterns.
- Single-file crate: concentrate profiling on `src/lib.rs` copy pipeline.

## Suggested experiments
- Profile large copy patterns to measure IO overhead.
- Compare cache hit rates for unchanged copied files.

## Code pointers
- `crates/rspack_plugin_copy/Cargo.toml`
- `crates/rspack_plugin_copy/src/lib.rs`
- `crates/rspack_plugin_copy/**`
