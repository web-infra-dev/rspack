# rspack_plugin_case_sensitive

## Role
Case-sensitive path checks for module resolution.

## Profiling relevance
- Not visible in react-10k; hot on case-insensitive filesystems.
- Costs scale with number of resolved paths.

## Perf opportunities
- Cache case check results per path to avoid repeated filesystem lookups.
- Batch checks when resolving many modules in same directory.
- Skip checks when filesystem is known to be case-sensitive.
- Single-file crate: concentrate profiling on `src/lib.rs` path checks.

## Suggested experiments
- Measure case-sensitive check overhead on macOS/Windows.
- Compare cached vs non-cached path checks.

## Code pointers
- `crates/rspack_plugin_case_sensitive/Cargo.toml`
- `crates/rspack_plugin_case_sensitive/src/lib.rs`
- `crates/rspack_plugin_case_sensitive/**`
