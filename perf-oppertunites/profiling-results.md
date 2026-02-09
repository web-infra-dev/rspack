# Profiling Results (react-10k)

This file records the perf output from the `react-10k` workload run against the
profiling build of Rspack.

## Flat Hotspots (perf report)

Command:

```bash
/usr/lib/linux-tools-6.8.0-100/perf report -i ./perf.data \
  --stdio --no-children -g none --percent-limit 0.5
```

Output (excerpt):

```
# Samples: 35K of event 'task-clock:kuH'
# Overhead  Command       Shared Object              Symbol
     2.41%  tokio-1       [kernel.kallsyms]          [k] clear_page_erms
     1.21%  tokio-1       rspack.linux-x64-gnu.node  [.] mi_malloc_aligned
     0.85%  tokio-2       [kernel.kallsyms]          [k] clear_page_erms
     0.83%  tokio-0       [kernel.kallsyms]          [k] clear_page_erms
     0.73%  tokio-0       rspack.linux-x64-gnu.node  [.] mi_malloc_aligned
     0.68%  tokio-2       rspack.linux-x64-gnu.node  [.] mi_malloc_aligned
     0.68%  tokio-1       [kernel.kallsyms]          [k] do_user_addr_fault
     0.65%  tokio-1       rspack.linux-x64-gnu.node  [.] core::str::lossy::Utf8Chunks::next
     0.61%  tokio-3       rspack.linux-x64-gnu.node  [.] mi_malloc_aligned
     0.58%  tokio-1       rspack.linux-x64-gnu.node  [.] rspack_core::module_graph::rollback::overlay_map::OverlayMap::get
     0.53%  tokio-1       [kernel.kallsyms]          [k] _raw_spin_unlock_irqrestore
     0.51%  tokio-1       rspack.linux-x64-gnu.node  [.] _mi_free_delayed_block
```

## Interpretation Summary

| Hotspot | Why it matters | Follow-up |
| --- | --- | --- |
| `mi_malloc_aligned`, `_mi_free_delayed_block`, `clear_page_erms` | Allocation/zeroing dominate samples. | Focus on reducing transient allocations in module graph updates, codegen jobs, and loader buffers. |
| `core::str::lossy::Utf8Chunks::next` | `String::from_utf8_lossy` conversions in loader/content paths. | See `rspack_loader_runner::Content` and related loaders; prefer byte buffers or cached string views. |
| `OverlayMap::get` | Module graph rollback overlay lookup in hot path. | Reduce overlay churn, avoid overlay in full builds, or use more cache‑friendly data layout. |
| `do_user_addr_fault` | Page faults due to large allocations. | Consider arena allocation for graph nodes, reuse buffers, reduce per-module allocations. |

## Call Graph Limitations

Call‑graph reporting (`perf report -g graph`) repeatedly timed out because
`addr2line` could not read build‑id debug entries for the perf binary on this
kernel. The analysis therefore relies on the flat hotspot list above.
