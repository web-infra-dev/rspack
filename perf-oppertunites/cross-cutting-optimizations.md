# Cross-Cutting Optimization Opportunities

This document highlights systemic optimizations that apply across multiple
pipeline stages.

## 1) Allocation Pressure & Memory Zeroing

Perf shows heavy time in `mi_malloc_aligned` and `clear_page_erms`. These point
to large allocations and page zeroing. Opportunities:

- Reuse buffers for module sources, codegen outputs, and diagnostics.
- Introduce arena allocators for short-lived graph data structures.
- Reduce `String` cloning by using `Arc<str>` or `Cow<str>`.

## 2) String Conversions (UTF‑8 lossy)

`core::str::lossy::Utf8Chunks::next` indicates repeated `from_utf8_lossy` calls
in loader content handling. Mitigations:

- Keep content as bytes until strictly needed as UTF‑8.
- Cache decoded strings inside `Content` or store `Arc<str>`.
- Avoid lossy conversions in debug formatting unless tracing is enabled.

## 3) Parallelism Strategy

Rspack uses tokio and custom futures scopes. Opportunities:

- Use bounded task pools for loaders and codegen to prevent oversubscription.
- Coalesce small tasks (tiny modules) into batched jobs.
- Minimize per-task allocations by reusing job structures.

## 4) Hook/Plugin Dispatch Overhead

The hook system is flexible but can be costly when many hooks are empty:

- Add fast paths when hook has no taps.
- Avoid cloning large `Compilation` structures for hook calls.
- Prefer small structs in hook arguments.

## 5) Hashing & Caching Strategy

Hashing is pervasive (module, chunk, asset). Opportunities:

- Cache hash digests and reuse them across passes.
- Replace repeated hash computations with incremental updates.
- Use bitset‑style runtime requirement tracking where possible.

## 6) NAPI Boundary Costs

Binding calls can introduce overhead, especially for large datasets:

- Batch JS↔Rust interactions to reduce call count.
- Use zero-copy buffers for sources and assets.
- Avoid serializing large structures unless needed by JS plugins.

## 7) Diagnostics & Logging

Diagnostics should be cheap when there are no errors:

- Use lazy formatting or `if logger.is_enabled()` checks.
- Avoid converting large data into strings for info‑level logs.
