# rspack_plugin_remove_duplicate_modules

## Role
Deduplicate modules in the graph to reduce output size.

## Perf opportunities
- Use fingerprints to avoid deep comparisons of identical modules.
- Avoid scanning entire module graph when no changes detected.
- Cache dedupe results across incremental builds.

## Code pointers
- `crates/rspack_plugin_remove_duplicate_modules/**`
