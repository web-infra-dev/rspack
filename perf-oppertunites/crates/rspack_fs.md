# rspack_fs

## Role
File system abstraction used by resolver and loader pipelines.

## Perf opportunities
- Cache stat/read results for repeated lookups in a single build.
- Batch IO operations to reduce syscall overhead.
- Avoid unnecessary path conversions in hot loops.

## Code pointers
- `crates/rspack_fs/**`
