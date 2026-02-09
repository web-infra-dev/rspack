# rspack_plugin_rsdoctor

## Role
Rsdoctor integration for profiling and diagnostics.

## Perf opportunities
- Ensure rsdoctor hooks are disabled in normal builds.
- Avoid heavy data collection unless explicitly enabled.
- Stream diagnostic data instead of building huge in-memory structures.

## Code pointers
- `crates/rspack_plugin_rsdoctor/**`
