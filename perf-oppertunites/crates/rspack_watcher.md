# rspack_watcher

## Role
File watching subsystem for incremental and watch builds.

## Perf opportunities
- Debounce file events to reduce redundant rebuild triggers.
- Batch filesystem stat calls and coalesce path updates.
- Use watch backend filters to avoid reporting irrelevant paths.

## Code pointers
- `crates/rspack_watcher/**`
