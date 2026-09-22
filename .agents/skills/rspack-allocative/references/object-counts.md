# Measuring object populations

## Distinguish metrics

- **Byte weights:** allocative's visited object graph, including represented inline storage and owned allocations.
- **Collection entries:** `len()` or iterator cardinality, not necessarily unique objects.
- **Live objects:** unique instances within a stated scope at a capture point; shared references require deduplication.
- **Allocation events:** allocator calls over an interval, unavailable from byte snapshots or live-object counts alone.

A `Vec<Node>` may hold 10,000 nodes in one allocation; capacity may exceed length. Many `Arc<Node>` references can point to one object. Arena blocks contain many objects. Do not infer populations from bytes, capacity, reference counts, or folded lines.

## Extend coverage

Find current roots with `rg 'allocative::root|visit_root|visit_global_roots' crates`. Types reachable only from a local compiler need explicit visitation and valid `Allocative` implementations. The current revision derives `Allocative` on `Compiler` and `Compilation`; inspect older revisions before assuming the same coverage.

For a supported local root, pass a borrowed reference into a phase-local collector and call `FlameGraphBuilder::visit_root(&root)`. Derive or implement `Allocative` for necessary types, audit skipped fields/ownership, and use the resolved crate's API. Custom traversal must deduplicate shared pointees. Gate instrumentation with `#[cfg(allocative)]` / `#[cfg_attr(allocative, ...)]`. Do not clone large structures for profiling.

The workspace vendors `rspack-allocative` 0.3.5 in `crates/rspack_allocative`, with Rspack adapters, shared ownership traversal and coverage diagnostics. Its folded output still contains bytes only. Native module/dependency entry counters are separate from the visitor; extend explicit counters for new populations instead of interpreting byte frames as instances.

## Capture counts beside a snapshot

At the snapshot call site, borrow the compilation and collect counts before teardown at a stable boundary. Allocate **one snapshot ID** for both `N-phase.allocative` and `N-phase.counts.json`; do not increment separate counters. Compute diagnostic JSON outside byte traversal where possible so collector allocations do not pollute the workload view.

These APIs existed in the inspected revision; verify them in the selected checkout:

```rust
let graph = compilation.get_module_graph();
let module_entries = graph.modules_len();
let dependency_entries = graph.dependencies().count();
let module_graph_module_entries = graph.module_graph_modules().count();
```

These are measured **entries**, not proof of a process-wide object count. Use `unit: "entries"`. For concrete types, classify entries using an existing type discriminator into nonoverlapping buckets. To call them `objects`, establish one entry per owned object or deduplicate by stable ID within the scope. Do not add an overall module map's count to a cache holding references to the same modules.

For AST nodes, use an AST visitor and count each node variant once at a phase where that AST exists. Post-build snapshots may miss discarded ASTs. Use a separately named phase snapshot if needed. Do not extend AST lifetimes for measurement without disclosing changed retention.

When collection APIs lack the desired population, add a small cfg-gated counting method or traversal. Constructor/drop counters are a fallback: account for cloning and all construction paths, identify live/cumulative/peak values, and do not treat an atomic read during mutation as a stable census.

## Sidecar contract

The renderer accepts the following JSON. Numbers here are **illustrative**, not Rspack observations:

```json
{
  "snapshot": "0-build",
  "unit": "entries",
  "scope": "One compilation at the post-build snapshot, before compiler drop",
  "counts": [
    {
      "path": ["Compilation", "ModuleGraph", "module_entries"],
      "count": 120,
      "basis": "compilation.get_module_graph().modules_len()"
    },
    {
      "path": ["Compilation", "ModuleGraph", "dependency_entries"],
      "count": 450,
      "basis": "compilation.get_module_graph().dependencies().count()"
    }
  ]
}
```

- `snapshot` equals the byte file's stem, e.g. `0-build` for `0-build.allocative`.
- `unit` is `objects` (measured unique live objects) or `entries` (collection cardinalities). Use separate reports for different units/scopes.
- `scope` states population and phase; `basis` names the expression or deduplicated traversal that produced the number.
- Each `path` consists of nonempty labels without semicolons or newlines; each leaf is a nonoverlapping population. Do not include both a total and its component buckets.
- `count` is a nonnegative integer. Omitted populations are unmeasured. An empty array is not a measurement.

Keep raw values and provenance. The helper generates a count-weighted folded file and SVG, never a byte-to-object estimate. Describe totals as scoped population totals, not allocator block counts or retained bytes.
