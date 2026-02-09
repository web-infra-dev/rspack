# rspack_fs

## Role
File system abstraction used by resolver and loader pipelines.

## Profiling relevance
- Indirectly visible via IO and path handling in resolver/loader.
- Costs scale with file count and cache effectiveness.

## Perf opportunities
- Cache stat/read results for repeated lookups in a single build.
- Batch IO operations to reduce syscall overhead.
- Avoid unnecessary path conversions in hot loops.

## Suggested experiments
- Profile builds with different filesystem cache settings.
- Measure IO time vs cached stat/read usage.

## Code pointers
- `crates/rspack_fs/**`
