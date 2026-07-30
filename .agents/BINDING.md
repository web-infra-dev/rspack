# JavaScript Binding Guide

Use this guide for changes involving any of these paths:

- `packages/rspack/src/`
- `crates/node_binding/`
- `crates/rspack_binding_api/`
- `crates/rspack_napi/`
- JavaScript hooks, loaders, file-system callbacks, or native-backed graph objects

Read the contributor-facing
[`JavaScript binding architecture`](../website/docs/en/contribute/architecture/javascript-binding.md)
for the full current design and
[`JavaScript binding design debt`](../website/docs/en/contribute/architecture/javascript-binding-design-debt.md)
before changing ownership or lifetimes.

## Scope

`@rspack/core` is the public JavaScript API. `@rspack/binding` is a private implementation package.
Do not expose `@rspack/binding` as a supported user entry point, even when its generated
declarations export a class or function.

The binding has two directions:

```text
JavaScript -> Node-API conversion -> Rust compiler
Rust compiler -> thread-safe function -> JavaScript callback -> Rust result
```

An API design is incomplete until both directions, ownership, lifetime, error conversion, and
native/WASI behavior have been considered.

## Non-negotiable invariants

### Public and internal boundaries

- Preserve webpack-compatible behavior in `packages/rspack` unless a documented Rspack difference is
  intentional.
- Keep compatibility-only JavaScript behavior out of `rspack_core`.
- Keep the standalone `@rspack/binding` surface private and non-SemVer.
- Do not re-export an internal binding type publicly without reviewing its runtime behavior and
  lifetime.

### Lifetimes and ownership

- Every native-backed object needs an owner, stable identity, valid access window, and revocation
  rule.
- A JavaScript cache preserves object identity; it does not extend the Rust object's lifetime.
- Treat `Compilation`, `Module`, `Chunk`, graph objects, dependencies, and blocks as
  compilation-scoped unless a stronger contract is proven.
- Old watch compilations must never resolve to the latest compilation merely because an identifier
  or slot was reused.
- Do not send a borrowed Rust reference through an asynchronous thread-safe function.
- Values queued to JavaScript must be owned or represented by a validated, revocable handle.
- Do not add `unsafe impl Send` or `unsafe impl Sync` to a wrapper without documenting the owner,
  transfer path, exclusivity, revocation, and proof that queued conversion cannot outlive them.
- Do not create `&mut` access from an originally shared reference.
- Prefer closure-based access that returns owned values; do not return fabricated `'static`
  references.

### Threads and callbacks

- JavaScript values may only be accessed in their JavaScript environment.
- Rust worker tasks call JavaScript through `rspack_napi` thread-safe function abstractions.
- A thread-safe function queue can be non-blocking while the native compilation still waits for its
  result.
- Promise hooks must convert both synchronous throws and asynchronous rejection.
- Sync JavaScript APIs block the caller and must not hide unbounded work.
- Follow the repository concurrency rules: use `rayon` for synchronous CPU parallelism and
  `rspack_parallel` for async compilation orchestration. Do not introduce binding-local Tokio or
  Rayon mixing in core workflows.

### Performance

- Treat every Rust/JavaScript crossing as observable cost in hot hooks.
- Avoid per-element callbacks and repeated property getters when an owned batch can be returned.
- Do not add locks to normal Rust graph traversal to simplify a rarely used JavaScript callback.
  Pay synchronization at the binding boundary where possible.
- Decide whether a collection is live, cached, or materialized; document that decision.
- Avoid converting sources or stats fields that the caller did not request.
- Preserve skip and cache behavior for hook registration unless benchmarks justify a change.

### Generated files

- `crates/node_binding/napi-binding.d.ts` is generated. Do not edit it directly.
- Change Rust `#[napi]` annotations or `crates/node_binding/scripts/banner.d.ts`.
- `crates/node_binding/binding.d.ts` is the handwritten CJS/ESM interop wrapper.
- Generated native and WASI exports must stay aligned.

## Layer and file map

| Task                      | Start here                              | Usually also inspect                                   |
| ------------------------- | --------------------------------------- | ------------------------------------------------------ |
| Public Compiler lifecycle | `packages/rspack/src/Compiler.ts`       | `crates/rspack_binding_api/src/lib.rs`                 |
| Public Compilation API    | `packages/rspack/src/Compilation.ts`    | `crates/rspack_binding_api/src/compilation/`           |
| Module API or identity    | `packages/rspack/src/Module.ts`         | `crates/rspack_binding_api/src/module.rs`              |
| ModuleGraph API           | `packages/rspack/src/ModuleGraph.ts`    | `crates/rspack_binding_api/src/module_graph.rs`        |
| ChunkGraph API            | `packages/rspack/src/ChunkGraph.ts`     | `crates/rspack_binding_api/src/chunk_graph.rs`         |
| Chunk API                 | `packages/rspack/src/Chunk.ts`          | `crates/rspack_binding_api/src/chunk.rs`               |
| Stats                     | `packages/rspack/src/Stats.ts`          | `crates/rspack_binding_api/src/stats.rs`               |
| Resolver                  | `packages/rspack/src/Resolver.ts`       | `crates/rspack_binding_api/src/resolver.rs`            |
| JavaScript hook bridge    | `packages/rspack/src/taps/`             | `plugins/interceptor.rs`, `plugins/js_hooks_plugin.rs` |
| JavaScript loader bridge  | `packages/rspack/src/loader-runner/`    | `plugins/js_loader/`, core loader runner               |
| File-system adapter       | `packages/rspack/src/FileSystem.ts`     | `crates/rspack_binding_api/src/fs_node/`               |
| Source conversion         | `packages/rspack/src/util/source.ts`    | `crates/rspack_binding_api/src/source.rs`              |
| Async runtime or TSFN     | `crates/rspack_napi/src/runtime.rs`     | `threadsafe_function.rs`, `compiler_scoped_tsfn.rs`    |
| Binding package/types     | `crates/node_binding/`                  | build script, generated declarations, WASI wrappers    |
| Raw options               | `packages/rspack/src/config/adapter.ts` | `crates/rspack_binding_api/src/raw_options/`           |

## Current execution model

### Compiler creation

The public JavaScript `Compiler` is created first. Native creation is lazy in
`Compiler.#getInstance()`:

1. Check core/binding version compatibility.
2. Convert normalized options into `RawOptions`.
3. Attach JavaScript function and virtual-file references.
4. Create hook register functions.
5. Wrap file systems and resolver factory.
6. Construct native `JsCompiler`.

The JavaScript compiler keeps `RawOptions` alive because the native side intentionally avoids
turning that object into an accidental strong reference cycle.

The native `JsCompiler` owns a `ManuallyDrop<Compiler>`, compiler-scoped thread-safe functions,
caches, compiler context, and virtual file store. Explicit `close()` is the normal cleanup path.
`unsafeFastDrop` is internal and only valid when process exit will reclaim all state.

### Hook bridge

The hook path is split:

```text
public Hook
  -> packages/rspack/src/taps/* register function
  -> RegisterJsTapKind
  -> Rust hook interceptor
  -> ThreadsafeJsTap
  -> public Hook invocation
```

`JsHooksAdapterPlugin` must install every native interceptor. The interceptor asks JavaScript for
taps by stage range. Frequently invoked hooks can cache the returned tap list. Hook kinds not used
by JavaScript can be skipped through the non-skippable register set.

When changing a hook, verify:

- the public hook exists and has the correct Tapable type;
- the JavaScript tap adapter converts arguments and results;
- `RegisterJsTapKind` contains the hook;
- the Rust register definition has the correct argument, return, Promise, cache, and skip settings;
- `JsHooksAdapterPlugin` installs it;
- cache invalidation handles taps added across the compiler lifecycle;
- errors preserve the hook context.

### Compilation identity

Rust uses `JsCompilationWrapper` to cache a native `JsCompilation` per `CompilationId`. JavaScript
uses `Compiler.#bindingCompilationMap` to cache the public `Compilation` facade per native instance.

Do not collapse these two layers casually. They preserve different identities:

```text
Rust compilation identity -> native binding identity -> public facade identity
```

Current wrappers include pointer-based access and depend on cleanup. Any change must test old
compilations, watch rebuilds, compiler close, garbage collection, and parallel compilers.

### Module identity

`ModuleObject` is the Rust-to-JavaScript conversion type. It preserves one JavaScript module
instance per `(CompilerId, ModuleIdentifier)`.

Normal access resolves the module identifier in the active compilation. Some build and loader
callbacks currently use a raw pointer fallback while the module is outside `ModuleGraph`. This is
active design debt, not a reusable pattern. Do not add another pointer-backed call site without an
architecture review.

Cleanup by revoked module identifiers and compiler identifier must continue to make stale objects
fail rather than resolve a different module.

## Classify an API before implementing it

Choose one primary model:

| Model                      | Use when                                          | Main review question                                    |
| -------------------------- | ------------------------------------------------- | ------------------------------------------------------- |
| JavaScript facade          | Compatibility behavior does not need native state | Can it remain completely outside the binding?           |
| Owned DTO                  | The result is data, not identity                  | Is conversion bounded and are all fields needed?        |
| Identifier lookup          | Native state has stable identity in an owner      | What invalidates the identifier?                        |
| Native-backed live view    | Live identity and queries are required            | How is every access validated and revoked?              |
| Snapshot plus patch        | Callback needs reads and limited mutation         | Can mutation be expressed explicitly?                   |
| Callback-scoped capability | Off-owner live access is unavoidable              | How is capability, generation, and revocation enforced? |
| JavaScript adapter         | User implementation must be called from Rust      | What is the call frequency and async behavior?          |

Do not begin with `#[napi]` or a pointer. Begin with ownership and observable behavior.

## Change playbooks

### Add a binding-backed method

1. Confirm the public API contract and webpack behavior.
2. Decide the API model from the table above.
3. Identify the Rust owner and invalidation point.
4. Implement Rust conversion without returning borrowed references.
5. Add or update the JavaScript facade.
6. Test wrong compiler, old compilation, revoked object, and close behavior when relevant.
7. Update user implementation notes if lifetime or cost is visible.

### Add a hook

1. Define the native hook and its Rust arguments.
2. Add `RegisterJsTapKind`.
3. Define the register/interceptor conversion.
4. Install the interceptor in `JsHooksAdapterPlugin`.
5. Add the JavaScript tap adapter.
6. Decide `cache`, `skip`, sync, and Promise behavior explicitly.
7. Test stage ordering, no-tap fast path, errors, repeated builds, and cleanup.

### Add a callback in raw options

1. Prefer an owned serializable option if possible.
2. Store JavaScript references under compiler ownership.
3. Pass owned callback arguments through a thread-safe function.
4. Convert return values in the JavaScript environment.
5. Release the reference on compiler close and environment cleanup.
6. Measure frequency if the callback can run per module, dependency, chunk, or asset.

### Change a native-backed wrapper

Write an invariant comment or design note covering:

- owner;
- identity key;
- access and mutation capability;
- valid phases;
- cross-thread representation;
- revocation and ABA protection;
- JavaScript instance caching;
- behavior after close and rebuild.

If the wrapper contains `NonNull`, a raw pointer, `WeakRef`, `Reference`, `External`, a manual
lifetime, or unsafe `Send`/`Sync`, inspect every construction, conversion, update, and cleanup site.

## Validation matrix

Run builds before tests when code changes:

```bash
# Rust binding changes
pnpm run build:binding:dev

# JavaScript-only changes
pnpm run build:js

# Changes spanning both layers
pnpm run build:cli:dev
```

Relevant tests:

| Behavior                            | Location or command                                              |
| ----------------------------------- | ---------------------------------------------------------------- |
| Binding declaration type check      | `pnpm --filter @rspack/binding test`                             |
| Compiler and Compilation API        | `tests/rspack-test/compilerCases/`                               |
| Hook conversion and stages          | `tests/rspack-test/hookCases/`, `configCases/hooks/`             |
| TSFN lifecycle and GC               | `compilerCases/fixtures/tsfn-lifecycle/`                         |
| Binding garbage collection          | `configCases/binding-gc/`                                        |
| Module and chunk graphs             | `configCases/module-graph/`, `configCases/chunk-graph/`          |
| Loader concurrency and importModule | `configCases/loader-parallel/`, `loader-parallel-import-module/` |
| Watch invalidation                  | `tests/rspack-test/watchCases/`                                  |
| Native/WASI behavior                | binding native and WASI builds plus targeted browser tests       |

Tests involving garbage collection or thread-safe function cleanup should run in a separate process
when global state or `--expose-gc` is required.

## Documentation contract

When changing observable behavior:

- update `website/docs/{en,zh}/api/javascript-api/architecture.mdx` for user-facing lifetime or cost;
- update the relevant JavaScript API reference page;
- update the contributor architecture when ownership, threading, or call paths change;
- update the design-debt register when a compromise is added, changed, or removed;
- write an ADR for an accepted long-lived architecture decision;
- update this file only for AI routing, invariants, or modification recipes.

Do not copy historical spike conclusions into current-state documentation until the implementation
has landed.
