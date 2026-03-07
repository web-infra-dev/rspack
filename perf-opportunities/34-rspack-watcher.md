# rspack_watcher — Performance Opportunities

**Size**: 2,156 lines across 12 files  
**Role**: File system watcher for watch mode / HMR — detects file changes and triggers rebuilds  
**Impact**: Low for build performance, but critical for developer experience (watch-mode latency)

---

## Architecture

The watcher system has multiple components:

```
DiskWatcher (notify crate) → FsEvent stream → Analyzer → Trigger → Rebuild
                                                  ↑
                                              Scanner
                                              (initial)
```

- **DiskWatcher**: Uses the `notify` crate for OS-level file watching
- **Scanner**: Initial directory scan to establish watch baseline
- **Analyzer**: Debounces and filters events (e.g., ignore node_modules)
- **Trigger**: Determines when to fire a rebuild based on accumulated changes
- **PathManager**: Tracks which paths to watch and which to ignore

---

## Performance Concerns for Watch Mode

### 1. Event Debouncing

The analyzer debounces file system events to avoid triggering multiple rebuilds for rapid changes (e.g., IDE save-all). The debounce strategy directly affects how quickly the user sees changes:

**Opportunity**: Use adaptive debouncing — shorter debounce when few events arrive (fast feedback for single-file saves), longer when many events arrive (batch saves).

### 2. Ignore Pattern Matching

The `FsWatcherIgnored` system checks each file event against ignore patterns. With many watched directories and complex ignore patterns, this can add latency per event.

**Opportunity**: Pre-compile ignore patterns into a trie or prefix tree for O(path_length) matching instead of O(patterns × path_length).

### 3. Watch Path Accumulation

As modules are discovered during build, their paths are added to the watch list. At 10K modules, this means 10K+ watched paths.

The OS watcher (inotify on Linux, FSEvents on macOS) has limits:
- Linux: `fs.inotify.max_user_watches` defaults to ~65K
- macOS: FSEvents watches directories, not individual files

**Opportunity**: Watch directories instead of individual files (rspack already does this for most cases), but verify edge cases with symlinks and node_modules.

### 4. Rebuild Trigger → Compilation Handoff

After the watcher triggers, the rebuild involves:
1. Computing changed/deleted file sets
2. Creating new compilation with `Incremental::new_hot`
3. Transferring artifacts from old compilation
4. Running the full pass pipeline

**This is the critical path for HMR latency.** The watcher itself adds ~1-5ms overhead. The real latency is in the rebuild (analyzed in `29-deep-dive-module-factory-and-rebuild.md`).

---

## Summary

The watcher crate itself is well-optimized and not a significant performance concern. The HMR/rebuild latency is dominated by the compilation system, not the file watching system.

| # | Opportunity | Impact | Effort |
|---|-----------|--------|--------|
| 1 | Adaptive debouncing | Better DX (faster feedback) | Low |
| 2 | Pre-compiled ignore pattern trie | <1ms per event | Medium |
