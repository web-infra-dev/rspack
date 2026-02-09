# rspack_plugin_dll

## Role
DLL plugin support (manifest and reference handling).

## Profiling relevance
- Not visible in react-10k; hot when DLL manifests are used.
- Costs scale with manifest size and reference count.

## Perf opportunities
- Cache DLL manifest parsing across builds.
- Avoid repeated file IO for unchanged DLLs.
- Reuse resolved module mappings when DLL config is stable.

## Suggested experiments
- Profile builds with large DLL manifests.
- Measure cache hit rates for manifest parsing.

## Code pointers
- `crates/rspack_plugin_dll/**`
