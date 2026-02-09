# Remaining Crates — Performance Opportunities

This document covers smaller crates that individually have limited performance impact but collectively contribute to the overall system.

---

## rspack_error (~1,772 lines)

**Role**: Error handling — `Diagnostic`, `Result`, error formatting

**Observations**:
- Errors create `Diagnostic` objects with string fields (module identifier, location, message)
- Error formatting involves string allocation
- The error system supports warnings and errors with source code context

**Opportunity**: Minimal. Error creation is infrequent in the happy path. Lazy error message formatting (format only when displayed) could save allocations for warnings that are never printed.

**Estimated Gain**: Negligible

---

## rspack_hook (~200 lines)

**Role**: Plugin hook system — `define_hook!` macro, hook types (Series, SeriesBail, Sync)

**Observations**:
- Hooks use dynamic dispatch via trait objects
- Each hook call involves iterating all tapped plugins
- The `define_hook!` macro generates hook structs with `tap()` and `call()` methods
- Some hooks have `tracing=false` for hot paths

**Opportunity**:
1. **Early bail for empty hooks**: Skip hook invocation entirely when no plugins are tapped
2. **Specialized hooks**: For hooks with a single tap (common for builtin plugins), use a direct function call instead of iterating

**Estimated Gain**: 1-2% of compilation time (hooks are called very frequently)

---

## rspack_ids (~1,557 lines)

**Role**: Module and chunk ID assignment — deterministic ID generation

**Observations**:
- Uses `assign_deterministic_ids` which sorts and assigns IDs
- Module IDs can be natural (readable) or deterministic (short)
- Chunk IDs follow similar patterns

**Opportunity**: The ID assignment algorithms are O(n log n) due to sorting. For 10K modules this is fast. No significant opportunities.

**Estimated Gain**: Negligible

---

## rspack_regex (~600 lines)

**Role**: Regex wrapper for matching module rules

**Observations**:
- Uses `regex` crate with lazy compilation (`LazyLock<Regex>`)
- Regex patterns are compiled once and reused
- Module rule matching checks each module against configured patterns

**Opportunity**: Regex performance is generally good. For very common patterns (like `/\.tsx?$/`), a specialized string check would be faster than regex.

**Estimated Gain**: <1%

---

## rspack_tracing (~200 lines)

**Role**: Performance tracing — Perfetto and stdout tracers

**Observations**:
- Uses `tracing-subscriber` with env filter
- Perfetto trace output for Chrome devtools
- Bench preset only enables info-level spans on `rspack_compilation_main` target

**Opportunity**:
1. **Zero-cost tracing**: Ensure tracing spans are compiled out when tracing is disabled (use `#[instrument]` conditionally)
2. **Reduced span overhead**: Some `#[instrument]` annotations on hot functions add overhead even when no subscriber is active

**Estimated Gain**: 1-2% when tracing is disabled (if not already zero-cost)

---

## rspack_util (~2,447 lines)

**Role**: Utility functions — `itoa`, `atom`, `source_map`, `ext`, `fx_hash`, `property_access`, `swc` helpers

**Observations**:
- `itoa::Buffer` is used for fast integer-to-string conversion (good)
- `fx_hash` provides `FxIndexMap` and `FxIndexSet` type aliases
- Various helper functions for string manipulation

**Opportunity**: Most utilities are well-optimized. The `compile_boolean_matcher` utility could potentially use a more efficient pattern matching approach.

**Estimated Gain**: Negligible

---

## rspack_fs (~1,367 lines)

**Role**: File system abstraction — `ReadableFileSystem`, `WritableFileSystem`, `IntermediateFileSystem`

**Observations**:
- 48 `#[instrument]` annotations on the NativeFileSystem — these add tracing overhead
- File I/O operations are the primary bottleneck in resolution and emission

**Opportunity**:
1. **Buffer pool for file reads**: Reuse read buffers instead of allocating new ones for each file
2. **Batch file operations**: When emitting assets, batch write operations to reduce syscall overhead
3. **Reduce tracing on hot paths**: The 48 instrument annotations on native FS operations add overhead

**Estimated Gain**: 1-3% of file I/O operations

---

## rspack_paths (~200 lines)

**Role**: Path type wrappers — `ArcPath`, `ArcPathSet`, `Utf8Path` utilities

**Observations**: Thin wrappers, no performance concerns.

**Estimated Gain**: Negligible

---

## rspack_tasks (~100 lines)

**Role**: Task spawning utilities — `spawn_in_compiler_context`, `CompilerContext`

**Observations**: The compiler context is passed through `tokio::task_local!` for all spawned tasks. This adds a small overhead per task spawn.

**Opportunity**: Consider using a lighter-weight context passing mechanism for hot task spawning paths.

**Estimated Gain**: <1%

---

## rspack_plugin_devtool (~2,106 lines)

**Role**: Source map generation — SourceMapDevToolPlugin, EvalSourceMapDevToolPlugin

**Observations**:
- Source map generation involves JSON serialization and string manipulation
- For large projects, source map generation can be significant

**Opportunity**: Parallelize source map generation across chunks. Use streaming JSON serialization.

**Estimated Gain**: 5-15% of source map generation time

---

## rspack_plugin_html (~1,767 lines)

**Role**: HTML generation — HtmlRspackPlugin

**Observations**: HTML generation is typically fast and runs once per entry. Not a performance concern.

**Estimated Gain**: Negligible

---

## Other Plugin Crates

The following crates are small, focused plugins with minimal performance impact:
- `rspack_plugin_banner` (801 lines) — Banner insertion
- `rspack_plugin_copy` (801 lines) — File copying
- `rspack_plugin_progress` (633 lines) — Build progress
- `rspack_plugin_ignore` — Module ignoring
- `rspack_plugin_asset` (999 lines) — Asset modules
- `rspack_plugin_json` — JSON modules
- `rspack_plugin_wasm` — WebAssembly modules
- Various library plugins (ESM, CommonJS, AMD, etc.)

These are not performance bottlenecks.

---

## Summary of Remaining Crates

| Crate | Top Opportunity | Estimated Gain |
|-------|----------------|----------------|
| rspack_hook | Early bail for empty hooks | 1-2% |
| rspack_tracing | Zero-cost tracing when disabled | 1-2% |
| rspack_fs | Buffer pool for file reads | 1-3% |
| rspack_plugin_devtool | Parallel source map generation | 5-15% of source maps |
