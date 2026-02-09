# rspack_plugin_rsdoctor

## Role
Rsdoctor integration for profiling and diagnostics.

## Profiling relevance
- Only active when rsdoctor is enabled.
- Potentially high data volume in large projects.

## Perf opportunities
- Ensure rsdoctor hooks are disabled in normal builds.
- Avoid heavy data collection unless explicitly enabled.
- Stream diagnostic data instead of building huge in-memory structures.

## Suggested experiments
- Measure rsdoctor overhead on large builds with profiling enabled.
- Validate streaming vs batch data collection impact.

## Code pointers
- `crates/rspack_plugin_rsdoctor/Cargo.toml`
- `crates/rspack_plugin_rsdoctor/src/lib.rs`
- `crates/rspack_plugin_rsdoctor/**`
