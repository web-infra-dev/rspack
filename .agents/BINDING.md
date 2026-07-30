# JavaScript Binding Guide

Use this guide for changes involving any of these paths:

- `packages/rspack/src/`
- `crates/node_binding/`
- `crates/rspack_binding_api/`
- `crates/rspack_napi/`
- JavaScript hooks, loaders, file-system callbacks, or native-backed graph objects

Read the contributor-facing
[`JavaScript API architecture`](../website/docs/en/api/javascript-api/architecture.mdx)
for the full current design before changing ownership or lifetimes.

## Scope

The binding has two directions:

```text
JavaScript -> Node-API conversion -> Rust compiler
Rust compiler -> thread-safe function -> JavaScript callback -> Rust result
```

An API design is incomplete until both directions, ownership, lifetime, error conversion, and
native/WASI behavior have been considered.

## Non-negotiable invariants

### Public and internal boundaries

Preserve webpack-compatible behavior in `packages/rspack` unless a documented Rspack difference is intentional.

### Lifetimes, ownership, and asynchronous access

Rust's lifetime and borrowing rules stop at the Node-API boundary. The lifetime model depends on
the kind of value exposed to JavaScript:

- A plain N-API object, such as an owned `#[napi(object)]` struct containing strings, numbers, arrays,
  or other owned values, is materialized as an ordinary JavaScript object. It is a snapshot owned by
  JavaScript after conversion. Dropping the Rust value used to create it does not invalidate normal
  JavaScript property access, and changing the JavaScript object does not mutate the original Rust
  value unless another binding call explicitly reads it back.
- An N-API class instance has different semantics. Node-API keeps the Rust wrapper associated with
  the JavaScript object alive, but that wrapper may contain only an identifier, weak owner
  reference, or pointer to a `Compilation`, `Module`, or another separately owned native value.
  Keeping the JavaScript object alive does not necessarily keep that target alive. Each getter,
  setter, or method call re-enters Rust and can occur after the target has been removed, dropped, or
  revoked.

For an N-API class, the binding must validate or re-resolve the native target on every getter,
setter, and method call before dereferencing it. If the target is no longer available, the
operation must fail with a clear error, as access through a stale `Module` instance does today. A
setter must additionally verify that mutation is allowed in the current compilation phase.

- Every native-backed class needs a Rust owner, stable identity, valid access window, permitted
  operations, and revocation rule.
- Treat `Compilation`, `Module`, `Chunk`, graph objects, dependencies, and blocks as
  compilation-scoped.
- After a compilation rebuild, object removal, compiler close, or owner drop, fail with a
  deterministic error. Never dereference a stale pointer or silently attach an old wrapper to a
  different native object whose identifier was reused.
- Prefer a plain owned object when the API only needs a snapshot. When live native behavior is
  required, prefer closure-based access that returns owned snapshots or identifiers.

Rspack deliberately does not emulate Rust's shared and exclusive borrow checking for native-backed
JavaScript classes. Checking every getter, mutation, and callback would add overhead to hot APIs and
would not match the object model expected by webpack-compatible plugins. The current contract
instead relies on hook phases and loader conventions: read and mutate binding-backed class
instances only while the Rspack-invoked hook or loader is active, and only through operations valid
for that phase. Being inside the access window is necessary, but does not make every mutation valid
in every hook.

The execution window depends on how Rspack invokes JavaScript:

- A synchronous tap is active until its callback returns.
- A Promise tap is active until the Promise returned by the tap settles; the native hook bridge
  awaits that Promise.
- A synchronous loader is active until it returns. An asynchronous loader remains active until its
  returned Promise settles or the callback obtained from `this.async()` is called; the native
  loader scheduler awaits the JavaScript loader runner.
- A timer, microtask, event listener, or Promise that is scheduled but not returned or otherwise
  connected to one of those completion mechanisms is detached work. It must not retain and later
  access native-backed class instances from the completed invocation.

For example, this code lets a native-backed `Module` class instance escape the hook that supplied
it:

```js
compilation.hooks.buildModule.tap('Plugin', (module) => {
  setTimeout(() => {
    module.identifier();
  }, 0);
});
```

Capture an owned value before returning instead:

```js
compilation.hooks.buildModule.tap('Plugin', (module) => {
  const identifier = module.identifier();
  setTimeout(() => consume(identifier), 0);
});
```

`setTimeout` is not inherently invalid. It is supported when the surrounding asynchronous API
keeps the invocation open. For example, this loader does not finish until `callback` is called:

```js
module.exports = function loader(source) {
  const callback = this.async();

  setTimeout(() => {
    this.addDependency('generated-dependency.js');
    callback(null, source);
  }, 0);
};
```

### Compiler-scoped thread-safe functions

A Node-API thread-safe function (TSFN) keeps a strong reference to its JavaScript callback. The
callback closure can in turn retain arbitrary JavaScript objects, preventing them from being
garbage-collected while the TSFN is alive. If those captured objects retain a `Compiler`,
`Compilation`, or another object that leads back to the native compiler, a cross-runtime ownership
cycle is formed:

```text
Rust Compiler -> TSFN -> JavaScript callback closure
      ^                         |
      |                         v
native binding <- JS Compiler or Compilation
```

JavaScript GC cannot see that releasing the Rust-owned TSFN would break the cycle,
while Rust cannot drop the compiler because it is still reachable from JavaScript. The compiler,
its compilation data, the callback, and everything captured by the callback can therefore remain
alive indefinitely.

`CompilerScopedTsFnHandle`, implemented in `compiler_scoped_tsfn.rs`, gives every compiler-owned
TSFN an explicit release boundary:

- `JsCompiler` owns one `CompilerScopedTsFnManager`.
- Raw options, built-in plugins, and hook registration callbacks are converted inside
  `CompilerScopedTsFnManager::scope`. The scope uses thread-local context because `FromNapiValue`
  cannot receive the owning compiler as additional conversion context.
- Each handle stores an `Arc<AtomicRefCell<Option<ThreadsafeFunction>>>`. Handle clones share the
  same slot. The manager registers a releaser that replaces the option with `None`, which drops the
  TSFN and its strong reference to the JavaScript callback for every clone at once.
- `JsCompiler::close` waits until in-flight build or rebuild work is idle before closing the native
  compiler, because that work may still need its callbacks. The close Promise releases the manager
  in `finally`; manager `Drop` performs the same release as a fallback.
- Calling a released handle fails with the compiler-closed error rather than invoking JavaScript
  after close.

All callbacks owned for the compiler lifetime must use `CompilerScopedTsFnHandle` rather than keep
an independent raw `ThreadsafeFunction`. After `compiler.close()` settles, no registered TSFN may
retain its JavaScript closure. Keeping another raw TSFN clone or strong function reference outside
the manager defeats this guarantee and can reintroduce the memory leak.

#### Defer conversion of callback-bearing values

The `#[napi]` macro normally runs `FromNapiValue` before entering the exported Rust method. This is
too early for a callback-bearing argument when the owning compiler is discovered from another
argument or from the `Compilation`: its TSFN would be created before
`CompilerScopedTsFnManager::scope` is active.

Use a two-phase conversion for values such as custom runtime modules that are attached after the
compiler has been constructed:

1. Accept the argument as an `Unknown` wrapper around the raw `napi_value`, and use
   `ts_args_type` to keep the intended public TypeScript signature.
2. Resolve the owning `JsCompiler` before converting the argument.
3. Enter that compiler's `CompilerScopedTsFnManager::scope`, then explicitly call
   `FromNapiValue::from_napi_value`.
4. Store every callback in the converted value as a `CompilerScopedTsFnHandle`, never as a raw
   `ThreadsafeFunction`.

```rust
#[napi(ts_args_type = "value: JsCallbackValue")]
pub fn add_value<'a>(
  &mut self,
  env: &'a Env,
  value: Unknown<'a>,
) -> napi::Result<()> {
  let value = js_compiler.compiler_scoped_tsfn_manager.scope(|| unsafe {
    JsCallbackValue::from_napi_value(env.raw(), value.raw())
  })?;

  // Store `value`; its callback fields use `CompilerScopedTsFnHandle`.
  Ok(())
}
```

### Performance

Binding performance is often dominated by how much data is converted and how many Node-API
operations are needed to construct the JavaScript result. Apply these two optimizations first:

1. **Choose eager properties and lazy getters by access pattern.** For a large native structure,
   expose expensive or rarely used fields through getters instead of converting the entire
   structure eagerly. A field that JavaScript never reads should incur no conversion cost.
   Conversely, if a field is likely to be read and its value is immutable in Rspack, convert it
   once while constructing the class instance and define it as an own JavaScript data property,
   for example with `Property::with_value`. This avoids a getter call, native-target validation,
   and repeated conversion on the common path. It also leaves JavaScript with an owned value that
   remains readable after the native target is revoked. Use a getter when the field is expensive
   and cold, or when it must reflect live native state.

   Keep laziness coarse-grained: when a getter returns a collection, convert that collection as one
   batch rather than introducing one Rust-to-JavaScript call per element. Decide whether each
   getter returns a live view, a newly materialized snapshot, or a cached snapshot, and document its
   invalidation behavior. Because a getter re-enters Rust, native-backed class lifetime and
   revocation rules still apply.

2. **Use JSON for large plain data objects.** For a large, JSON-compatible struct with no native
   identity or behavior, the fastest transfer path is generally:

   ```text
   Rust value -> JSON string -> one Node-API string transfer -> JSON.parse in JavaScript
   ```

   This avoids constructing a large object graph through many individual Node-API property and
   value conversions. Use direct N-API conversion for small objects, and benchmark when the
   threshold matters: serialization, parsing, and the temporary string also have costs. Do not use
   this path when the API must preserve `undefined`, `BigInt`, functions, symbols, cyclic
   references, prototypes, class identity, typed binary data, or other values that JSON cannot
   represent faithfully.

Treat every Rust/JavaScript crossing as observable cost in a hot hook. Batch at a meaningful API
boundary, avoid per-element callbacks, and do not eagerly convert data merely because it is
available on the Rust side.

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
| Rspack options            | `packages/rspack/src/config/adapter.ts` | `crates/rspack_binding_api/src/raw_options/`           |

### N-API class property placement

N-API can expose the same JavaScript property syntax through descriptors in two different places.
The placement is observable and should be chosen as part of the API contract:

| Mechanism                                         | Descriptor location       | JavaScript behavior                                                                 |
| ------------------------------------------------- | ------------------------- | ----------------------------------------------------------------------------------- |
| `#[napi(getter)]` and `#[napi(setter)]`           | Class template/prototype  | Shared by instances, inherited through the prototype chain, and not an own property |
| `Object::define_properties` after `into_instance` | Individual class instance | Created for each instance and reported as an own property                           |

Prototype accessors are the conventional choice for class behavior. They avoid installing the same
descriptor on every instance, but `Object.hasOwn(instance, name)` returns `false`, and
`console.log(instance)` normally does not show inherited properties. This can make important
user-facing state difficult to discover while inspecting or debugging an object.

`Object::define_properties` allows the binding to install data or accessor descriptors directly on
the class instance. Own properties can appear in `console.log(instance)` and property enumeration
when their descriptor attributes make them enumerable. They are appropriate when own-property
behavior, inspection UX, or per-instance descriptor values are part of the API.

Descriptor location is independent of how the value is produced:

- `Property::with_getter` and `Property::with_setter` create accessors. Whether they are installed
  on the prototype or the instance, each access enters Rust and must follow native lifetime,
  revocation, and mutation rules.
- `Property::with_value` creates a data property from a value converted during instance
  construction. It is suitable for immutable, frequently accessed state that should remain owned
  and readable on the JavaScript side without another native call.

Rspack uses instance-level `define_properties` for N-API classes whose public state should be
visible during ordinary object inspection; `Module` and its derived classes are one application of
this general strategy. Use `#[napi(getter)]` or `#[napi(setter)]` when prototype semantics are
intended. Do not move a property between the prototype and the instance as a mechanical refactor:
it can change `console.log`, `Object.hasOwn`, `Object.keys`, inheritance, and
`Object.getOwnPropertyDescriptor` results.
