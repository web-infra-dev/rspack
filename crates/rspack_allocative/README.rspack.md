# Rspack allocative adapters

Based on the published `rspack-allocative` 0.3.5 crate (Meta's allocative, MIT OR Apache-2.0). The original source headers and licenses are retained. Local changes add adapters needed by Rspack and correct collection accounting. Debug (`release-debug`) and profiling binding builds include it automatically; other builds must opt in. Snapshot collection requires `RSPACK_ALLOCATIVE_DIR` at runtime.

Only adapters used by Rspack are retained. Unused upstream adapters, size-only visitors, and SVG/golden test helpers are removed. Rspack regression tests live in the dedicated `rspack_allocative_testing` crate:

```sh
cargo test -p rspack_allocative_testing --features allocative \
  --config 'target."cfg(all())".rustflags=["--cfg=allocative"]'
```

Most workspace types use `#[cfg_attr(allocative, derive(allocative::Allocative))]`. Dynamic ownership interfaces require `MaybeAllocative` only when profiling is enabled; this makes missing adapters a compile error. Explicit callback/third-party boundaries use `visit_opaque`, which emits a coverage warning. Do not replace a missing owning implementation with `size_of` or `skip`.

`FlameGraphBuilder::with_shared_ownership()` attributes shared payload to the first owner and terminates strong cycles. That owner may vary with hash iteration order. Compare represented totals or stable populations rather than assuming a shared allocation always appears under the same path.

The byte tree includes capacity where the container exposes it. Private hash-table/node metadata, allocator overhead and inaccessible external state remain outside complete accounting. It is an ownership estimate, not RSS, peak usage or an allocation-event trace. The compiler snapshot writes these limits and coverage warnings alongside the folded data. Counts are measured separately from the current module graph; folded byte weights never imply object counts.
