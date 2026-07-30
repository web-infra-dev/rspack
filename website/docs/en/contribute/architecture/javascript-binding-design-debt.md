---
description: 'Current compromises, risks, and exit criteria in the Rspack JavaScript binding architecture.'
---

# JavaScript binding design debt

This document records known architectural compromises in the JavaScript binding. It is not a
roadmap and does not mean that every item should be changed immediately. Each change must preserve
webpack compatibility, performance, object identity where required, and native/WASI support.

Keep historical investigation in `docs/spikes/` and accepted decisions in `docs/adr/`. This page
describes current state and should be updated when that state changes.

## Debt register

| Area                                  | Risk   | Current direction                                                                               |
| ------------------------------------- | ------ | ----------------------------------------------------------------------------------------------- |
| Callback-scoped Module access         | High   | Replace project-owned raw pointer access with revocable, capability-aware handles or owned data |
| Compilation wrapper pointer lifetime  | High   | Reduce pointer lifetime and resolve access through an owner-controlled context                  |
| Hook registration and callback caches | Medium | Centralize invalidation and make cache ownership observable                                     |
| Compiler cleanup and `unsafeFastDrop` | Medium | Make explicit close cheap enough to remain the default ownership path                           |
| Split type ownership                  | Medium | Clarify generated, handwritten internal, and public types                                       |
| Hidden boundary cost                  | Medium | Document and benchmark high-frequency JavaScript API calls                                      |
| Native and WASI parity                | Medium | Define capability differences and test both export surfaces                                     |

## Callback-scoped Module access

**Status:** Active

**Affected code:** `crates/rspack_binding_api/src/module.rs`, module hooks, loader context, module
factory callbacks, and asset callbacks.

**Current behavior:** A binding `Module` normally resolves `(compiler_id, module_identifier)` through
the active compilation. During phases where a module is owned outside `ModuleGraph`, `ModuleObject`
can carry a raw pointer fallback. The value is passed through the JavaScript callback bridge, and a
per-compiler instance cache preserves JavaScript identity.

**Why it exists:** `buildModule`, `succeedModule`, loaders, module factory callbacks, and module
execution can expose modules before or while they are outside the main graph. Identifier-only lookup
would make webpack-compatible `Module` APIs unavailable in those callbacks.

**Assumptions and risks:**

- JavaScript does not access the pointer-backed object after the native callback scope.
- A shared Rust reference is not used to enable JavaScript mutation.
- queued callback conversion and Promise completion do not outlive the native owner.
- manual `Send` and `Sync` contracts remain valid.

These assumptions are not fully represented by Rust types, and retained JavaScript objects can
violate them.

**Desired direction:** Use owned snapshots for genuinely read-only callbacks and revocable,
versioned, capability-aware handles for callbacks that require live access or mutation. Pay
synchronization at the JavaScript boundary, not in normal Rust graph traversal.

**Exit criteria:**

- project-owned Module raw pointers and manual `Send`/`Sync` are removed or encapsulated behind a
  reviewed abstraction;
- read-only callbacks cannot mutate through a shared reference;
- retained JavaScript objects fail deterministically after revocation;
- module identity and loader behavior remain compatible;
- lifecycle and concurrency tests cover Promise retention and rebuilds.

## Compilation wrapper pointer lifetime

**Status:** Active

**Affected code:** `JsCompilation`, `JsCompilationWrapper`, compilation instance caches, and
`COMPILER_REFERENCES`.

**Current behavior:** The native wrapper stores a `CompilationId` and a non-null pointer to the Rust
`Compilation`. Rust and JavaScript caches preserve one wrapper and one public facade for the active
compilation. Cleanup removes old cache entries.

**Why it exists:** Compilation APIs are broad and mutable. Copying the full compilation is neither
practical nor compatible, while repeated reconstruction would break object identity and add
conversion cost.

**Risk:** Safety depends on cleanup, compiler lifetime, and callback sequencing. The wrapper
representation expresses a longer and more transferable lifetime than the underlying borrow.

**Desired direction:** Route access through an owner-controlled compiler or compilation context,
with explicit invalidation and closure-based access that cannot return borrowed references.

**Exit criteria:**

- old watch compilations cannot resolve a new compilation accidentally;
- all access validates an owner and compilation generation;
- wrapper types no longer need an unconstrained pointer lifetime;
- compiler close and garbage collection have deterministic failure behavior.

## Hook registration and callback caches

**Status:** Active and performance-sensitive

**Affected code:** `packages/rspack/src/Compiler.ts`,
`crates/rspack_binding_api/src/plugins/interceptor.rs`, and `JsHooksAdapterPlugin`.

**Current behavior:** JavaScript exposes register functions. Native hook interceptors query taps by
stage, skip unused hook kinds, and cache tap lists for frequently invoked hooks. Some invalidation is
triggered after JavaScript tap execution.

**Why it exists:** Registering and converting JavaScript functions on every module or asset would
add significant overhead. Rspack also needs webpack-compatible hook stages without installing every
JavaScript tap as a permanent native object at startup.

**Risk:** Ownership and invalidation are distributed across JavaScript and Rust. Late taps can be
missed if a cache is stale, and cached thread-safe functions can retain compiler or compilation
closures.

**Desired direction:** Give registration snapshots an explicit generation and owner, centralize
invalidation, and expose counters or tracing for cache queries and callback invocation.

**Exit criteria:**

- cache invalidation rules are written as invariants and tested across rebuilds;
- callback handles are released on close and environment cleanup;
- adding a hook requires one declarative mapping rather than coordinated boilerplate in several
  files;
- hot-hook overhead has a regression benchmark.

## Compiler cleanup and `unsafeFastDrop`

**Status:** Active internal optimization

**Affected code:** JavaScript `Compiler.unsafeFastDrop`, native `JsCompiler` finalization, and
compiler-scoped thread-safe function management.

**Current behavior:** The native compiler is stored in `ManuallyDrop`. Normal cleanup explicitly
drops it. The internal fast-drop mode skips the expensive drop and relies on process teardown.

**Why it exists:** Dropping a large compiler graph at the end of short-lived CLI execution can be
visible in total command time.

**Risk:** Skipping destructors is only appropriate when the process is about to exit. Reuse in a
long-lived process would retain native memory and callback resources.

**Desired direction:** Keep the optimization internal, make its process-lifetime assumption
explicit, and reduce or move cleanup work so normal explicit ownership remains affordable.

**Exit criteria:**

- long-lived API users always use deterministic cleanup;
- tests demonstrate that callbacks and native state are released by `close()`;
- process-exit optimization cannot be enabled accidentally through the public API.

## Split type ownership

**Status:** Active

**Affected code:** `crates/node_binding/napi-binding.d.ts`,
`crates/node_binding/scripts/banner.d.ts`, `crates/node_binding/binding.d.ts`, Rust `#[napi]`
annotations, and public `@rspack/core` types.

**Current behavior:** napi-rs generates most internal declarations. A handwritten banner supplies
types that cannot be expressed conveniently by generation, and `binding.d.ts` fixes CJS/ESM
interop. The public package then re-exports or wraps selected binding types.

**Risk:** A type can appear correct in one layer while disagreeing with runtime conversion or the
public wrapper. Internal generated types can also be mistaken for supported public APIs.

**Desired direction:** Mark type ownership explicitly and add type tests at each boundary:

- Rust-generated internal binding surface;
- handwritten internal supplement;
- public `@rspack/core` API.

**Exit criteria:**

- generated files are never edited manually;
- each public binding-backed API has one authoritative public type;
- type tests cover CJS, ESM, native, and WASI entry points where applicable.

## Hidden boundary cost

**Status:** Under-documented

**Affected APIs:** Graph getters, module and chunk iteration, stats conversion, assets, sources,
custom file systems, loaders, and high-frequency hooks.

**Current behavior:** JavaScript syntax does not reveal whether an operation is pure JavaScript, a
native lookup, collection materialization, source conversion, or a Rust-to-JavaScript callback.

**Risk:** Plugins can accidentally turn a linear native phase into thousands of cross-language
calls or repeated graph scans.

**Desired direction:** Classify public APIs by boundary behavior, add implementation notes to API
reference pages, and benchmark representative operations rather than only full builds.

**Exit criteria:**

- high-frequency APIs document lifetime, materialization, mutation, and expected cost;
- hot binding operations have microbenchmarks or tracing;
- new public APIs include a boundary-cost review.

## Native and WASI parity

**Status:** Active

**Affected code:** native platform packages, `rspack.wasi.cjs`, browser wrappers, worker adapters,
and conditional Rust features.

**Current behavior:** The same generated API is adapted to native Node.js and WASI environments, but
runtime, file-system, worker, and feature behavior can differ.

**Risk:** A new export can compile and work natively while being unsupported, inefficient, or
incorrect under emnapi/WASI.

**Desired direction:** Maintain an explicit platform capability matrix and exercise the generated
export list in both native and WASI CI.

**Exit criteria:**

- unsupported APIs fail explicitly or are omitted intentionally;
- platform differences are documented in the Browser API;
- new boundary types are tested for WASI conversion and worker behavior.

## Updating this register

When a compromise is removed or changes shape:

1. Update the current architecture document first.
2. Move lasting rationale into an ADR.
3. Update this debt entry or remove it.
4. Update `.agents/BINDING.md` if an invariant or modification recipe changed.
5. Add or update tests that prove the exit criteria.
