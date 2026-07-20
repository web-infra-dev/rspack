# `import.meta.hot` Shallow Wrapper Implementation Plan

> **For agentic workers:** Follow test-driven development. Complete Tasks 1-4
> as one green atomic slice because the code-generation, facade, and apply
> changes must land together. Request a code review after Task 4 and again after
> final verification.

**Goal:** Replace the independent `import.meta.hot` registry with a shallow
facade backed by `module.hot`, reducing runtime size and duplicated lifecycle
logic while preserving the existing public context semantics.

**Architecture:** Compile `import.meta.hot` to `HOT_CONTEXT(module.hot)`. Cache
the facade and differential callback records on a non-enumerable private field
of the webpack hot object. Delegate self acceptance, data transport, and
disposal to webpack HMR. Keep dedicated apply behavior only for dependency
namespace callbacks, partial dependency failures, and self success callbacks.

**Tech stack:** Rust dependency/code generation, Rspack EJS runtime modules,
JavaScript HMR apply runtime, configCases, hotCases, runtime snapshots, and
existing TypeScript regression tests.

**Design:**
`docs/superpowers/specs/2026-07-20-import-meta-hot-module-wrapper-design.md`

## Global Constraints

- Preserve `module.hot` and `import.meta.webpackHot` public behavior exactly.
- `import.meta.hot` must remain a distinct object, not an alias.
- Keep `WEBPACK_HOT_CONTEXT`/`hmrW` and `HOT_CONTEXT`/`hmrH` names.
- Do not introduce implementation identifiers containing `Vite` or `vite`.
- Keep all existing dependency namespace, partial-error, data, dispose, and
  self-callback behavior.
- Do not change TypeScript declarations or `import.meta.glob`.
- Do not stage unrelated worktree files.
- Add a failing assertion before each production behavior change.
- Skip storage and native-watcher tests if they hang, as required by the
  repository instructions.

## Target Files

### Modify

- `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs`
- `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs`
- `crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs`
- `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`
- `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/index.js`
- `tests/rspack-test/hotCases/vite/import-meta-hot-self/index.js`
- `tests/rspack-test/hotCases/vite/import-meta-hot-self/self.js`
- `tests/rspack-test/hotCases/vite/import-meta-hot-self/mixed.js`
- affected HMR snapshots after intentional output changes

### Verify without production changes

- `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_accept_refresh.rs`
- `packages/rspack/module.d.ts`
- `packages/rspack/import-meta-hot.d.ts`
- `tests/type-tests/resolution-bundler/index.ts`
- `website/docs/en/api/runtime-api/hmr.mdx`
- `website/docs/zh/api/runtime-api/hmr.mdx`

---

## Task 1: Lock the Shallow-wrapper Code-generation Contract

**Files:**

- Modify: `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`
- Test: `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs`

- [ ] **Step 1: Replace the independent-registry source assertion**

Update the configCase so it requires the generated context call to receive the
current module hot object rather than the module id. Match the actual rendered
module argument used by the fixture, and assert all of these semantic facts:

```js
expect(source).toContain('.hmrH(');
expect(source).toContain('.hot)');
expect(source).not.toContain('.hmrH(module.id)');
expect(import.meta.hot).toBe(import.meta.hot);
expect(import.meta.hot).not.toBe(module.hot);
```

Retain the assertions that webpack-only members such as `decline` and `status`
are absent from the facade.

- [ ] **Step 2: Run the focused configCase and confirm RED**

```bash
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: FAIL because the current output calls `hmrH(module.id)`.

- [ ] **Step 3: Record the current feature-runtime baseline**

Build the current branch and record byte counts for the generated web bundle
and the relevant runtime snapshot before modifying production code:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "hotCases/vite/import-meta-hot"
wc -c js/runtime-mode-hot-web/vite/import-meta-hot-dependency/bundle.js
wc -c hotCases/vite/import-meta-hot-dependency/__snapshots__/web/0.snap.txt
```

Store the two numbers in the task notes; do not add a generated report file.

- [ ] **Step 4: Continue without committing**

The test is intentionally red until Tasks 2-4 replace the facade and apply
integration together.

---

## Task 2: Add Data, Dispose, and Mixed-registration Regressions

**Files:**

- Modify: `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/index.js`
- Modify: `tests/rspack-test/hotCases/vite/import-meta-hot-self/index.js`
- Modify: `tests/rspack-test/hotCases/vite/import-meta-hot-self/self.js`
- Modify: `tests/rspack-test/hotCases/vite/import-meta-hot-self/mixed.js`

- [ ] **Step 1: Cover repeated and mixed dependency registrations**

Extend `import-meta-hot-dependency/index.js` with one dependency that is
accepted by:

- two separate `import.meta.hot.accept` calls;
- one `module.hot.accept` call in the same accepting module.

After the update, assert that both context callbacks receive the updated
namespace exactly once and the webpack callback still receives its existing
outdated-id argument exactly once.

- [ ] **Step 2: Cover context-data identity and hidden storage**

Extend the self fixture to retain the first `import.meta.hot.data` reference.
After an accepted update assert:

```js
expect(import.meta.hot.data).toBe(initialData);
```

When `module.hot.data` is available in the replacement evaluation, assert that
one of its own property values is the same context-data object, while its
enumerable keys contain only user-owned webpack data. Use
`Object.getOwnPropertyNames` to inspect ownership without asserting the private
key spelling. This assertion must fail while data is still stored only in the
independent `hotData` map.

- [ ] **Step 3: Make single self evaluation explicit**

Increment an evaluation counter in `mixed.js`. For both a successful update
and the existing throwing update, assert that mixed webpack/context self
acceptance evaluates the new factory at most once. Retain the assertions that
the context success callback does not run after a failed evaluation.

- [ ] **Step 4: Run the focused hotCases and confirm RED where appropriate**

```bash
cd tests/rspack-test
pnpm run test -t "hotCases/vite/import-meta-hot-dependency"
pnpm run test -t "hotCases/vite/import-meta-hot-self"
```

Expected: the new data-storage assertion fails against the independent
registry. Existing semantic assertions must remain green; the code-generation
assertion from Task 1 remains red.

- [ ] **Step 5: Continue without committing**

Keep Tasks 1-2 as the failing test half of the atomic slice.

---

## Task 3: Replace the Registry with a `module.hot` Facade

**Files:**

- Modify: `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs`
- Modify: `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs`

- [ ] **Step 1: Generate `HOT_CONTEXT(module.hot)`**

In `ImportMetaHotDependency::hash`, replace the
`RuntimeGlobals::MODULE_ID` contribution with `RuntimeGlobals::MODULE`.

In the dependency template, keep rendering the current module argument but
replace the source with:

```rust
format!("{getter}({module_argument}.hot)")
```

Insert only:

```rust
RuntimeGlobals::HOT_CONTEXT | RuntimeGlobals::MODULE
```

into runtime requirements.

- [ ] **Step 2: Rewrite `hot_context.ejs` as a facade factory**

Remove `hotContexts`, `hotData`, the module-id argument, `.get`, and `.dispose`.

Implement this behavior:

1. Return the existing private facade when the hot object already has one.
2. Recover context data from a non-enumerable private slot on
   `moduleHot.data`, or create `{}`.
3. Create private state containing `data`, `acceptCallbacks`, and
   `selfCallback`.
4. Define the state/facade on `moduleHot` as non-enumerable.
5. Register one internal webpack dispose handler that persists the same context
   data object into the outgoing webpack data object through a non-enumerable
   private slot.

- [ ] **Step 3: Delegate compatible methods**

Implement facade methods as follows:

```text
accept()             -> moduleHot.accept()
accept(callback)     -> moduleHot.accept(); save selfCallback
accept(dep, callback)  -> save one differential dependency record
accept(deps, callback) -> save one differential dependency record
dispose(callback)    -> moduleHot.dispose(wrapper passing context data)
```

Preserve the existing optional refresh argument injected by the compiler for
dependency forms. Do not call `moduleHot.accept(deps, callback)`, because that
would overwrite mixed or repeated registrations.

- [ ] **Step 4: Keep the runtime conditional**

Do not change `HotContextRuntimeModule` registration in
`crates/rspack_plugin_hmr/src/lib.rs`: `hmrH` must still be installed only when
`RuntimeGlobals::HOT_CONTEXT` is requested.

- [ ] **Step 5: Build and run the configCase**

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: code generation passes. Update hotCases may remain red until Task 4
switches apply state lookup.

---

## Task 4: Reuse webpack Apply and Lifecycle State

**Files:**

- Modify: `crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs`
- Test: all files changed in Tasks 1-2

- [ ] **Step 1: Read context state directly from `module.hot`**

Replace every weak runtime-global lookup of this form:

```js
var hotState = HOT_CONTEXT ? HOT_CONTEXT.get(moduleId) : undefined;
```

with a null-safe read from the already cached module:

```js
var hotContext = module && module.hot._hotContext;
```

Use the actual private property defined by Task 3. Apply the same rule to
parent modules.

- [ ] **Step 2: Preserve dependency propagation and callbacks**

Continue treating a matching context dependency record as an acceptance
boundary. Snapshot `acceptCallbacks` from the accepting module's old hot object
before disposal, and keep the current namespace/refresh/error-continuation
callback loop.

Do not alter the webpack `_acceptedDependencies` callback loop or error-handler
arguments. Both loops must run when both API families accept the same child.

- [ ] **Step 3: Delete dedicated disposal**

Remove the call to `HOT_CONTEXT.dispose(moduleId, removed)`. Context dispose
callbacks and data persistence now run inside `module.hot._disposeHandlers`.

- [ ] **Step 4: Collapse self acceptance into one work item**

Remove the separate context self-accepted entry. The existing webpack entry is
created because the facade called `module.hot.accept()`.

Add the old context success callback to that entry. Re-evaluate the module once:

- on success, obtain the new namespace and call the context callback;
- on failure, run the existing webpack self-error handler/reporting path and do
  not call the context callback.

Delete `selfEvaluationResults`, the `type: "import-meta-hot"` branch, and the
extra `require(moduleId)` path.

- [ ] **Step 5: Remove apply-time `HOT_CONTEXT` requirements**

After all lookups are removed, ensure the generic JavaScript HMR runtime no
longer references weak `HOT_CONTEXT`. `HOT_CONTEXT` remains required only by
modules whose generated source evaluates `import.meta.hot`.

- [ ] **Step 6: Build and run the atomic slice**

```bash
pnpm run build:cli:dev
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
pnpm run test -t "hotCases/vite/import-meta-hot"
pnpm run test -t "hotCases/esm-dependency-import/import-meta-webpack-hot"
pnpm run test -t "hotCases/runtime/accept"
pnpm run test -t "hotCases/runtime/self-accept-and-dispose"
pnpm run test -t "hotCases/recover/recover-after-self-error"
```

Expected: all PASS.

- [ ] **Step 7: Inspect identifiers and the focused diff**

```bash
rg -n "hotContexts|hotData|HOT_CONTEXT.*get|HOT_CONTEXT.*dispose" \
  crates/rspack_plugin_hmr/src/runtime \
  crates/rspack_plugin_runtime/src/runtime_module/runtime
rg -n -i "vite" \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs \
  crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs
git diff --check
```

Expected: both searches return no implementation hits and `git diff --check`
passes.

- [ ] **Step 8: Commit the green atomic slice**

Stage only the production files and focused tests changed by Tasks 1-4:

```bash
git add \
  crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs \
  tests/rspack-test/configCases/parsing/import-meta-hot/index.js \
  tests/rspack-test/hotCases/vite/import-meta-hot-dependency \
  tests/rspack-test/hotCases/vite/import-meta-hot-self
git commit -m "refactor: wrap module hot for import meta hot"
```

- [ ] **Step 9: Request focused code review**

Review the commit for webpack callback compatibility, mixed registrations,
single self evaluation, data identity, private-property visibility, and removal
of parallel lifecycle state. Resolve Critical and Important findings before
continuing.

---

## Task 5: Update Runtime Snapshots and Measure Size

**Files:**

- Modify only snapshots changed by the intentional runtime output refactor.

- [ ] **Step 1: Run snapshot suites without update mode**

```bash
cd tests/rspack-test
pnpm exec rstest HotSnapshot.hottest.js
pnpm exec rstest StatsAPI.test.js
pnpm exec rstest StatsOutput.test.js
```

Expected: only runtime text and deterministic hashes affected by this refactor
may fail.

- [ ] **Step 2: Inspect every failure before accepting it**

For each changed snapshot verify:

- `hmrH` receives `module.hot`;
- no registry or dedicated dispose helper remains;
- webpack-only output does not acquire `hmrH`;
- changes outside runtime text are only expected hash references.

- [ ] **Step 3: Update only approved snapshots**

Run `-u` separately for the exact failing suites, then rerun each without `-u`.
Do not bulk-update unrelated snapshots.

- [ ] **Step 4: Record the after-size comparison**

Repeat the Task 1 byte-count commands. Compare them with the recorded baseline.
The feature bundle and representative snapshot must be no larger. If either
grows, inspect the emitted code and reduce the facade/apply implementation
before continuing.

- [ ] **Step 5: Commit snapshot updates**

```bash
git add tests/rspack-test
git commit -m "test: update import meta hot wrapper snapshots"
```

Stage only inspected snapshot files. If no snapshots changed, skip this commit.

---

## Task 6: Verify Types, Documentation, and Public Boundaries

**Files:**

- Verify without modification unless an incorrect internal claim is found:
  `packages/rspack/module.d.ts`
- Verify without modification unless an incorrect internal claim is found:
  `packages/rspack/import-meta-hot.d.ts`
- Verify without modification unless an incorrect internal claim is found:
  `website/docs/en/api/runtime-api/hmr.mdx`
- Verify without modification unless an incorrect internal claim is found:
  `website/docs/zh/api/runtime-api/hmr.mdx`

- [ ] **Step 1: Confirm the public contract did not change**

```bash
git diff 7e0cc38e81 -- packages/rspack/module.d.ts \
  packages/rspack/import-meta-hot.d.ts \
  tests/type-tests/resolution-bundler/index.ts
```

Expected: no changes.

- [ ] **Step 2: Check documentation wording**

Keep wording that `import.meta.hot` is a distinct API context and not an alias.
If documentation claims that it owns an independent module-id registry, update
only that internal statement to describe the shallow facade.

- [ ] **Step 3: Run type and package verification**

```bash
pnpm run test:type
pnpm run check-dependency-version
```

Expected: PASS. Do not add a committed `vite/client` compatibility fixture and
do not change `import.meta.glob`.

- [ ] **Step 4: Commit documentation only if needed**

```bash
git add website/docs/en/api/runtime-api/hmr.mdx \
  website/docs/zh/api/runtime-api/hmr.mdx
git commit -m "docs: describe import meta hot wrapper runtime"
```

Skip the commit when no public documentation change is necessary.

---

## Task 7: Full Verification and Final Review

- [ ] **Step 1: Build the final JavaScript and Rust integration**

```bash
pnpm run build:cli:dev
```

- [ ] **Step 2: Run repository-required tests**

```bash
pnpm run test:rs
pnpm run test:unit
```

Skip storage or native-watcher tests only if they hit the repository's known
sandbox hang; record the exact skipped target.

- [ ] **Step 3: Run lint and formatting checks**

```bash
pnpm run lint:js
pnpm run lint:rs
cargo lint
cargo fmt --all --check
```

- [ ] **Step 4: Inspect repository state and final diff**

```bash
git diff --check
git status --short
git diff 7e0cc38e81..HEAD --stat
git diff 7e0cc38e81..HEAD -- \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs \
  crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs
```

Expected: no unintended files, no implementation identifier containing the
ecosystem brand, and no public webpack or type changes.

- [ ] **Step 5: Request final branch review**

The reviewer must specifically check:

- no unintended breaking change to webpack HMR;
- the facade is distinct from `module.hot`;
- no parallel module-id registry remains;
- repeated and mixed dependency callbacks are preserved;
- dependency failures continue with aligned `undefined` values;
- mixed self acceptance evaluates once;
- runtime byte counts do not increase;
- types and `import.meta.glob` are untouched.

- [ ] **Step 6: Report completion**

Report focused/full verification results, before/after byte counts, review
findings, final commits, and any intentionally untracked local compatibility
package. Do not push or create a pull request unless explicitly requested.

---

## Review Checkpoints

1. After Task 4: focused runtime/codegen review before snapshot acceptance.
2. After Task 5: snapshot and byte-size review.
3. After Task 7: final branch review and breaking-change assessment.

## Expected Commit Sequence

1. `docs: design import meta hot module wrapper`
2. `refactor: wrap module hot for import meta hot`
3. `test: update import meta hot wrapper snapshots` (only if needed)
4. `docs: describe import meta hot wrapper runtime` (only if needed)
