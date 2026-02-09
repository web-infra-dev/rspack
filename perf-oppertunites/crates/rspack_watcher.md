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

## Key functions/structs to inspect
- `FsWatcher::new` (lib.rs) — initializes disk watcher, executor, scanner.
- `FsWatcher::watch` (lib.rs) — main loop for event aggregation.
- `FsWatcher::close` / `wait_for_event` (lib.rs) — shutdown and event handling.

## Suggested experiments
- Run watch mode on large repos and measure rebuild triggers with/without debouncing.
- Compare event volume across different watcher backends.

## Code pointers
- `crates/rspack_watcher/Cargo.toml`
- `crates/rspack_watcher/src/lib.rs`
- `crates/rspack_watcher/src/disk_watcher.rs`
- `crates/rspack_watcher/src/executor.rs`
- `crates/rspack_watcher/src/analyzer/mod.rs`
- `crates/rspack_watcher/**`
