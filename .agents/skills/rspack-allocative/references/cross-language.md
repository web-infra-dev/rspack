# Cross-language report checks

Read when interpreting combined Rust/JavaScript profiles or investigating unmatched objects.

Use the `.js.allocative` file for JavaScript memory and `.combined.allocative` for the combined profile. The `.links.json` report helps locate Rust objects associated with JavaScript objects. A matched object is not necessarily measured: check `measured` and the accompanying warnings before drawing conclusions about native memory.

Unmatched objects may fall outside the capture's coverage. Report them as unmeasured rather than assuming zero bytes or a match based only on the type name. When a retained object appears after a rebuild, do not assume it represents the previous build's data.

Validate these totals:

- JS category totals equal `represented_bytes`.
- `represented_bytes + excluded_v8_bytes = snapshot_bytes`.
- Combined bytes equal Rust bytes plus represented JS bytes.

Converted profiles exclude V8 internals. To inspect those, open the original `.heapsnapshot` in Chrome DevTools.

Path subtotals describe attributed memory, not retained size or object counts. Shared memory is counted once; its attributed path can vary between captures. Compare matching workloads and phases, and do not add parent and child inclusive totals.

Before sharing, remember that raw snapshots may contain source code and project data. Converted profiles omit string contents but retain object names by default. Use `--clear-class-names` to remove custom constructor names; built-in names and Rust type names remain. Inspect reports before sharing them.
