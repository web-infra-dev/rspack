# `import.meta.hot` Shallow Wrapper Design

## Status

Approved for implementation planning on 2026-07-20.

This design supersedes the independent-state architecture described in
`docs/superpowers/plans/2026-07-17-vite-import-meta-hot-runtime.md`. The public
API contract introduced by that work remains unchanged; only the internal
runtime ownership model changes.

## Goal

Make `import.meta.hot` a shallow wrapper around the existing webpack HMR
lifecycle instead of maintaining a parallel module-id keyed runtime. Reuse
`module.hot` wherever the two APIs have compatible semantics, and retain extra
runtime behavior only where the `import.meta.hot` contract differs.

The refactor must achieve both of these outcomes:

- reduce generated HMR runtime code;
- reduce duplicated state, disposal, propagation, and self-evaluation logic.

## Non-goals

- `import.meta.hot` does not become an alias of `module.hot`.
- The public behavior or types of `module.hot` and `import.meta.webpackHot` do
  not change.
- No new hot-context methods are added.
- `import.meta.glob` declarations and compatibility are not changed.
- The opt-in `@rspack/core/import-meta-hot` type entry is not redesigned.

## Naming

- `RuntimeGlobals::WEBPACK_HOT_CONTEXT` remains the webpack hot-object factory
  and renders as `hmrW`.
- `RuntimeGlobals::HOT_CONTEXT` remains the `import.meta.hot` facade factory and
  renders as `hmrH`.
- New implementation identifiers must not contain `Vite` or `vite`.
- Compatibility-facing prose and the existing compatibility test grouping may
  continue to use the ecosystem name.

## Architecture

`module.hot` is the sole owner of the module's HMR lifecycle. The generated
expression for `import.meta.hot` changes from a module-id lookup to a wrapper
around that hot object:

```text
module execution
    |
    +-- WEBPACK_HOT_CONTEXT(module.id, module)
    |       `-- creates module.hot and webpack HMR state
    |
    `-- HOT_CONTEXT(module.hot)
            `-- returns a memoized import.meta.hot facade
```

`HOT_CONTEXT` remains a conditionally installed runtime global, but it no
longer owns a module registry. Its only responsibilities are to memoize the
facade, translate API calls, preserve context data through webpack disposal,
and record callbacks whose semantics cannot be represented by webpack's
public callback contract.

The following independent runtime state is removed:

- `hotContexts`;
- `hotData`;
- `HOT_CONTEXT.get(moduleId)`;
- `HOT_CONTEXT.dispose(moduleId, removed)`.

## Private State

The facade and its differential state are attached to the current `module.hot`
object through a private, non-enumerable property. Conceptually the value is:

```text
module.hot._hotContext
    facade
    data
    dependencyAcceptRecords
    selfCallback
```

The exact property spelling is an internal implementation detail. It must be
non-enumerable so webpack users do not observe it through normal property or
data enumeration.

The state is recreated with each webpack hot object. State that must cross an
accepted update is carried through `module.hot.data`, not through a global
module-id map.

## Code Generation

`ImportMetaHotDependencyTemplate` renders the current module argument's hot
object:

```js
__webpack_require__.hmrH(module.hot);
```

The dependency requires `RuntimeGlobals::HOT_CONTEXT` and
`RuntimeGlobals::MODULE`; it no longer requires `RuntimeGlobals::MODULE_ID`.
Production folding and the independent `importMeta.hot` parser flag remain
unchanged.

Repeated `import.meta.hot` expressions during one module execution return the
same facade, but that facade is never identical to `module.hot`.

## API Mapping

### `data`

The facade always exposes an object through `data`.

On first execution, the wrapper creates that object. During disposal, an
internal `module.hot.dispose` handler stores the same object in a private,
non-enumerable slot on the webpack dispose-data object. The next hot object
recovers the context data from `module.hot.data`.

This preserves the context-data object identity across accepted updates while
leaving the webpack dispose-data object and its enumerable user fields
unchanged.

### `dispose(callback)`

Each context dispose callback is registered through `module.hot.dispose` as a
small wrapper that passes the context-data object instead of webpack's
dispose-data object. The facade does not maintain a separate dispose callback
array, and the apply runtime does not invoke a dedicated context-disposal
method.

### Self accept

Both `accept()` and `accept(callback)` call `module.hot.accept()` without a
callback. This reuses webpack's self-accept propagation marker without
mistaking the context callback for webpack's self-error handler.

For `accept(callback)`, the success callback is stored in the private context
state. The apply runtime creates one self-accepted work item per module. That
item may contain both:

- webpack's existing self-error handler;
- the context facade's success callback.

The module is evaluated exactly once. On success, the context callback receives
the new module namespace. On failure, the context callback is not called and
the existing webpack error-handler/reporting behavior runs unchanged.

### Dependency accept

Dependency accept cannot be delegated directly to
`module.hot.accept(dependencies, callback)` because webpack callbacks receive
outdated dependency ids, registrations for the same dependency overwrite one
another, and mixed webpack/context registrations must coexist.

The facade therefore stores only these differential records on
`module.hot._hotContext`:

```text
dependencies
callback
refresh closure
single-or-array result shape
```

The existing weak dependency parsing and refresh closure remain unchanged.
The HMR propagation and apply runtime read the records directly from
`module.hot`; there is no runtime-global lookup.

Before disposing an outdated accepting module, apply snapshots its dependency
records. After updated factories are installed, each affected record:

1. evaluates every declared dependency in declaration order;
2. reports an evaluation error without stopping later dependencies;
3. places `undefined` in the failed dependency's result position;
4. runs the existing refresh closure;
5. calls the user callback once with either one namespace or an aligned array.

Multiple context registrations for one dependency all run. A webpack accept
callback and a context accept callback for the same dependency also both run.

## Apply Runtime Changes

The JavaScript HMR apply runtime reads differential state from the cached
module's hot object:

```js
var hotContext = module.hot._hotContext;
```

It no longer uses weak `HOT_CONTEXT` references to query or dispose state.

Propagation still treats a dependency record as an acceptance boundary. The
dependency namespace callback loop remains because its semantics are
different. Self acceptance, disposal, module cache removal, error handling,
and re-evaluation otherwise use the webpack path.

The current duplicate self work items and `selfEvaluationResults` deduplication
map are removed. A mixed webpack/context self-accepted module has one work item
and one evaluation.

## Error Semantics

- A failed dependency evaluation is reported and produces `undefined` for that
  dependency; later dependencies and records continue.
- A dependency callback exception is reported through the existing apply
  `reportError` path.
- A self-accepted module callback runs only after successful evaluation.
- A self-evaluation failure continues to use webpack's self-error handler and
  `onErrored` behavior.
- Exceptions from context dispose callbacks follow the existing webpack
  dispose-handler behavior because they execute in that handler list.

## Compatibility

The refactor preserves:

- webpack HMR callback arguments and error-handler behavior;
- `module.hot` and `import.meta.webpackHot` object identity and methods;
- the four supported `import.meta.hot.accept` forms;
- context `data` and `dispose` behavior;
- parser option folding and runtime opt-in;
- current TypeScript declarations and documentation surface.

The private facade state is non-enumerable and is never part of a public type.
No webpack callback is wrapped, overwritten, or given additional arguments.

## Runtime-size Requirement

The dedicated registry and its lifecycle methods must disappear from generated
runtime code. Representative HMR output must not grow relative to the current
branch, and the implementation report must record the before/after byte counts
for at least one bundle using `import.meta.hot`.

Webpack-only bundles must not contain `hmrH`. The generic HMR apply runtime may
contain the small private-state checks required to recognize context records,
but it must not contain weak `HOT_CONTEXT` lookups.

## Testing Strategy

Add or strengthen tests for:

- `hmrH(module.hot)` code generation and non-alias object identity;
- repeated facade access during one execution;
- persistent data identity and non-enumerable internal storage;
- multiple context callbacks accepting the same dependency;
- mixed webpack/context acceptance of the same dependency;
- ordered namespace arrays with `undefined` on partial failure;
- self success, self failure, and mixed self acceptance with one evaluation;
- absence of `hmrH` from webpack-only bundles;
- absence of `hotContexts`, `hotData`, `.get`, and dedicated `.dispose` from
  generated runtime snapshots.

Run the existing webpack HMR suites as regression coverage. Types and
`import.meta.glob` remain out of the production change and are verified only to
confirm that the refactor did not alter them.

## Acceptance Criteria

The design is complete when all of the following are true:

1. `import.meta.hot` is a distinct facade backed by `module.hot`.
2. No module-id keyed context or data registry remains.
3. Only dependency namespace callbacks and self success callbacks have
   dedicated apply behavior.
4. Mixed webpack/context registrations retain both contracts.
5. Runtime output is smaller or equal for the representative feature bundle.
6. Focused hot cases, webpack regressions, type tests, Rust tests, unit tests,
   lint, and formatting checks pass.
