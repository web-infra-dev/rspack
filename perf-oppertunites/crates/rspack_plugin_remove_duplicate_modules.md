# rspack_plugin_remove_duplicate_modules

## Role
Deduplicate modules in the graph to reduce output size.

## Profiling relevance
- Not visible in react-10k perf samples; hot for graphs with many similar modules.
- Cost scales with number of candidate duplicates.

## Perf opportunities
- Use fingerprints to avoid deep comparisons of identical modules.
- Avoid scanning entire module graph when no changes detected.
- Cache dedupe results across incremental builds.

## Suggested experiments
- Profile a build with many identical modules and compare fingerprint strategies.
- Measure dedupe pass time across incremental builds.

## Code pointers
- `crates/rspack_plugin_remove_duplicate_modules/**`
