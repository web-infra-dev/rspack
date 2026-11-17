# Measuring object populations

## Distinguish metrics

- **Byte weights:** memory represented in the snapshot.
- **Collection entries:** the number of entries in a collection, not necessarily unique objects.
- **Live objects:** unique instances within a stated scope at a capture point, counting shared objects once.
- **Allocation events:** allocator calls over an interval, unavailable from byte snapshots or live-object counts alone.

A collection can contain many objects, and several entries can refer to the same object. Do not infer object counts from memory size or folded lines.

## Use measured counts

The native snapshot includes module and dependency entry counts for the current compilation. These are **entries**, not a process-wide object census. Keep the matching `.counts.json` beside its `.allocative` file when preparing reports.

For additional populations, use measurements from the same workload and capture point as the byte snapshot. Record the population, phase, scope, unit, and measurement basis. Report missing populations as unmeasured, not zero. A post-build snapshot cannot describe objects already released before capture.

Use `entries` for collection cardinalities. Use `objects` only for unique live instances within the stated scope, counting shared objects once. Do not add a population to another population that already includes it.

## Custom count file

The helper accepts the following JSON. Numbers here are **illustrative**, not Rspack observations:

```json
{
  "snapshot": "0-build",
  "unit": "entries",
  "scope": "One compilation at the post-build snapshot, before compiler drop",
  "counts": [
    {
      "path": ["Compilation", "ModuleGraph", "module_entries"],
      "count": 120,
      "basis": "Measured module entries in this compilation"
    },
    {
      "path": ["Compilation", "ModuleGraph", "dependency_entries"],
      "count": 450,
      "basis": "Measured dependency entries in this compilation"
    }
  ]
}
```

- `snapshot` equals the byte file's stem, e.g. `0-build` for `0-build.allocative`.
- `unit` is `objects` (measured unique live objects) or `entries` (collection cardinalities). Use separate reports for different units/scopes.
- `scope` states population and phase; `basis` describes how the number was measured.
- Each `path` consists of nonempty labels without semicolons or newlines; each leaf is a nonoverlapping population. Do not include both a total and its component buckets.
- `count` is a nonnegative integer. Omitted populations are unmeasured. An empty array is not a measurement.

Keep raw values and provenance. The helper generates a count-weighted folded file for Speedscope, never a byte-to-object estimate. Describe totals as scoped population totals, not allocator block counts or retained bytes.
