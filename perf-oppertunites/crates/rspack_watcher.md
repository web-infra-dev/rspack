# rspack_watcher

## Role
File watching subsystem for incremental and watch builds.

## Profiling relevance
- Not present in single build perf samples; hot in watch mode.
- Key for large projects with many file events.

## Perf opportunities
- Debounce file events to reduce redundant rebuild triggers.
- Batch filesystem stat calls and coalesce path updates.
- Use watch backend filters to avoid reporting irrelevant paths.

## Suggested experiments
- Run watch mode on large repos and measure rebuild triggers with/without debouncing.
- Compare event volume across different watcher backends.

## Code pointers
- `crates/rspack_watcher/**`
