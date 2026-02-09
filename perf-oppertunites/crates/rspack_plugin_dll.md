# rspack_plugin_dll

## Role
DLL plugin support (manifest and reference handling).

## Perf opportunities
- Cache DLL manifest parsing across builds.
- Avoid repeated file IO for unchanged DLLs.
- Reuse resolved module mappings when DLL config is stable.

## Code pointers
- `crates/rspack_plugin_dll/**`
