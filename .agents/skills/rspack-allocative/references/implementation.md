# Capture implementation reference

Read when extending instrumentation, diagnosing missing captures, or comparing revisions with different capture behavior. Paths below are relative to the selected Rspack checkout.

Check these integration points in the selected revision:

- `crates/node_binding/scripts/build.mjs`: debug (`release-debug`) and profiling builds enable the Cargo feature, `--cfg=allocative`, and the allocative compiler driver automatically; `ALLOCATIVE` opts other binding builds in.
- `scripts/allocative-driver.mjs`: builds the host driver with the pinned nightly's `rustc-dev` and wraps Cargo for Rust-only profiling. Ordinary source enums are instrumented in memory; macro-generated and borrowed/const-array enums retain explicit adapters.
- `crates/rspack_binding_api/src/lib.rs`: `RunGuard::finish` captures the live Rust `JsCompiler` on the JS thread after the build/rebuild future has ended, before releasing its running guard and strong reference.
- `crates/rspack_allocative/src/napi_tracking.rs` and `napi_shim.rs`: the optional `napi-tracking` feature owns class interception, weak registrations, target links, and coordinated capture. The binding enables it only through its `allocative` feature; the separate `napi` feature only provides type traversal adapters.
- `crates/rspack_allocative/src/snapshot.rs`: the optional `snapshot` feature owns output reservation, global-root/Ustr traversal, and file writing. `crates/rspack_binding_api/src/allocative/mod.rs` supplies module/dependency graph entry counts; core has no snapshot output logic.
- `scripts/heapSnapshot.mjs` in this skill: offline streaming parsing and V8 graph helpers used by `v8-to-allocative.mjs`. Its `stream-json` and `stream-chain` dependencies are repository-root dev dependencies; install them from `RSPACK_ROOT`. The runtime package only marks objects and writes raw V8 snapshots.
- Cargo manifests: the actual allocative package/version and feature propagation.

Node captures visit **Rust JsCompiler → compiler → core Compiler → Compilation**, including the wrapper's other fields, registered global roots, and the process-global Ustr string arena. The capture runs after cache finalization and after the build future releases its mutable compiler borrow, before the JS completion callback and compiler destruction. GC and V8 serialization finish synchronously on the owning JS thread before native traversal. Core build/rebuild has no automatic capture entry point; all automatic captures start in the binding. Shared allocations are charged once, below their first encountered owner; later references and cycles do not add their payload again. Hash iteration can change the first-owner path between runs. Read the snapshot metadata and warnings before interpreting a total. Older revisions visited global roots only; inspect the checkout before comparing results.
