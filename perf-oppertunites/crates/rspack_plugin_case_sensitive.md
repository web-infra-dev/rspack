# rspack_plugin_case_sensitive

## Role
Case-sensitive path checks for module resolution.

## Perf opportunities
- Cache case check results per path to avoid repeated filesystem lookups.
- Batch checks when resolving many modules in same directory.
- Skip checks when filesystem is known to be case-sensitive.

## Code pointers
- `crates/rspack_plugin_case_sensitive/**`
