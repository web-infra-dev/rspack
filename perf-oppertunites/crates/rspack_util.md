# rspack_util

## Role
Utility helpers used across the codebase (path, string, tracing helpers).

## Profiling relevance
- Utilities appear indirectly via path conversion, hashing, and logging.
- Hot when used inside module graph and resolver loops.

## Perf opportunities
- Ensure hot helpers avoid allocation; prefer `Cow<str>`/borrowed slices.
- Cache frequently used conversions (path normalization, hashing).
- Avoid regex use in hot paths; replace with lightweight parsers.

## Suggested experiments
- Track allocations in utility helpers during module graph build.
- Benchmark path normalization helpers with large inputs.

## Code pointers
- `crates/rspack_util/Cargo.toml`
- `crates/rspack_util/src/lib.rs`
- `crates/rspack_util/src/atom.rs`
- `crates/rspack_util/src/fx_hash.rs`
- `crates/rspack_util/src/node_path.rs`
- `crates/rspack_util/src/tracing_preset.rs`
- `crates/rspack_util/**`
