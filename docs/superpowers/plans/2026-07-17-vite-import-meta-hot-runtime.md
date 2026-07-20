# Vite-Compatible `import.meta.hot` Runtime Implementation Plan

> **Historical plan:** This plan describes the implementation already present
> at commit `7e0cc38e81`. Its independent module-id registry architecture and
> the final instruction forbidding `module.hot`-backed state are superseded by
> `docs/superpowers/specs/2026-07-20-import-meta-hot-module-wrapper-design.md`
> and its corresponding implementation plan. The public API requirements in
> this document remain relevant.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the webpack alias implementation of `import.meta.hot` with a dedicated runtime that implements Vite-compatible `accept` callback semantics without changing `module.hot` or `import.meta.webpackHot`.

**Architecture:** Compile `import.meta.hot` to a new `RuntimeGlobals::HOT_CONTEXT` getter keyed by module ID. Keep webpack’s existing hot-object factory under the explicit `RuntimeGlobals::WEBPACK_HOT_CONTEXT` name. The dedicated context runtime owns per-module state (`data`, self-accept callbacks, dependency-accept records, and dispose handlers), while the JavaScript HMR apply runtime consults that state alongside—but independently from—the existing webpack `module.hot` state. Dependency requests are still statically resolved into weak module dependencies, but the dedicated runtime invokes callbacks with updated module namespaces instead of webpack outdated-ID arrays.

**Tech Stack:** Rust parser/dependency/code-generation plugins, Rspack EJS runtime modules, JavaScript HMR runtime, TypeScript ambient declarations, Rspack configCases and hotCases.

## Global Constraints

- Do not change the behavior or public types of `module.hot` or `import.meta.webpackHot`.
- `import.meta.hot` must never be code-generated as `module.hot` or `module.hot.*`.
- Internal Rust, EJS, and emitted-JavaScript identifiers must not contain `Vite`/`vite`. Brand names are allowed only in compatibility-facing prose, test grouping, and the `vite/client` coexistence test.
- Use `RuntimeGlobals::WEBPACK_HOT_CONTEXT` for the webpack `module.hot` factory and `RuntimeGlobals::HOT_CONTEXT` for the dedicated `import.meta.hot` registry. Render them as `hmrW` and `hmrH`, respectively.
- Support the four Vite `accept` forms exactly: `accept()`, `accept(cb)`, `accept(dep, cb)`, and `accept(deps, cb)`.
- A single-dependency callback receives `ModuleNamespace | undefined`; an array callback receives an array aligned with the declared dependency list.
- A self-accept callback runs after successful module re-evaluation and receives the new module namespace; it is not an error handler.
- Preserve `import.meta.hot.data` across accepted updates and run `dispose(cb)` before replacing the module.
- Preserve the existing production folding and independent `module.parser.javascript.importMeta.hot` / `webpackHot` flags.
- Vite client-transport APIs (`on`, `off`, and `send`) and export-boundary APIs (`acceptExports` and `prune`) are outside this plan; types and documentation must not claim them. They require separate dev-server/export-propagation designs.
- Follow TDD: add or tighten a failing test before each production-code change.
- Skip storage and native-watcher tests as required by the repository instructions.

---

## File Structure

### Create

- `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs` — presentational dependency that replaces `import.meta.hot` with the new runtime getter.
- `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_accept.rs` — weak module dependency for Vite dependency-accept requests.
- `crates/rspack_plugin_hmr/src/hot_context.rs` — Rust `RuntimeModule` wrapper for the Vite context EJS template.
- `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs` — per-module Vite HotContext registry and public `accept`/`dispose`/`data` implementation.
- `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/index.js` — single- and multi-dependency namespace callback assertions.
- `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/dep.js` — single dependency update fixture.
- `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/a.js` — first array dependency fixture.
- `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/b.js` — second array dependency fixture.
- `tests/rspack-test/hotCases/vite/import-meta-hot-self/index.js` — driver for self-accept and lifecycle assertions.
- `tests/rspack-test/hotCases/vite/import-meta-hot-self/self.js` — self-updating module fixture.
- `packages/rspack/import-meta-hot.d.ts` — opt-in global `ImportMeta.hot` declaration for projects that do not load `vite/client`.
- `.superpowers/sdd/vite-hot-context-compat/` — local, uncommitted package that verifies the supported hot-context subset against the Vite hot-context type.

### Modify

- `crates/rspack_core/src/runtime_globals.rs` — add `WEBPACK_HOT_CONTEXT` as `hmrW` and `HOT_CONTEXT` as `hmrH`.
- `crates/rspack_core/src/dependency/dependency_type.rs` — rename the webpack dependency kind to `ImportMetaWebpackHotAccept` and add `ImportMetaHotAccept`.
- `crates/rspack_plugin_javascript/src/dependency/hmr/mod.rs` — export the new dependencies.
- `crates/rspack_plugin_javascript/src/parser_plugin/hot_module_replacement_plugin.rs` — rename the existing webpack-only parser to `ImportMetaWebpackHotReplacementParserPlugin` and add a separate `ImportMetaHotReplacementParserPlugin`.
- `crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs` — register the new dependency templates.
- `crates/rspack_plugin_javascript/src/visitors/dependency/util.rs` — rename webpack expression constants with a `WEBPACK` segment, give the unprefixed constants to `import.meta.hot`, and remove `.decline` handling from the dedicated context.
- `crates/rspack_plugin_hmr/src/runtime/hot_module_replacement.ejs` — expose the unchanged webpack hot-object factory as `WEBPACK_HOT_CONTEXT`.
- `crates/rspack_plugin_hmr/src/lib.rs` — register both parser plugins, dependency factories, and the conditional runtime module.
- `crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs` — consult dedicated accept state during propagation, disposal, dependency callback execution, and self re-evaluation.
- `packages/rspack/module.d.ts` — remove `hot?: Rspack.Hot` and export the Vite-shaped namespace types.
- `packages/rspack/package.json` — export and publish `./import-meta-hot`.
- `tests/type-tests/resolution-bundler/index.ts` — verify all supported callback overloads.
- `tests/rspack-test/configCases/parsing/import-meta-hot/index.js` — assert dedicated-runtime code generation.
- `tests/rspack-test/configCases/parsing/import-meta-hot/parser-options.js` — ensure `hot: false` still preserves the native expression.
- `website/docs/en/api/runtime-api/hmr.mdx` — document the split webpack/Vite contracts and examples.
- `website/docs/zh/api/runtime-api/hmr.mdx` — mirror the English documentation.

---

### Task 1: Lock the Non-Alias Code-Generation Contract

**Atomic slice:** Tasks 1–4 are one TDD batch. Intermediate focused tests are expected to stay red; do not commit until Task 4 installs the runtime and makes the complete context slice pass.

**Files:**

- Modify: `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`
- Modify: `tests/rspack-test/configCases/parsing/import-meta-hot/parser-options.js`
- Modify: `crates/rspack_core/src/runtime_globals.rs`
- Create: `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_context.rs`
- Modify: `crates/rspack_plugin_javascript/src/dependency/hmr/mod.rs`
- Modify: `crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs`

**Interfaces:**

- Produces: `RuntimeGlobals::WEBPACK_HOT_CONTEXT`, rendered as `hmrW`, for the existing webpack hot-object factory.
- Produces: `RuntimeGlobals::HOT_CONTEXT`, rendered as `hmrH`, for the dedicated `import.meta.hot` registry.
- Produces: `ImportMetaHotDependency::new(range, loc)` replacing the source range with `<require>.hmrH(<module>.id)`.
- Consumes: the current module argument rendered by `RuntimeTemplate::render_module_argument`.

- [ ] **Step 1: Tighten the configCase so the old alias fails**

Add these assertions to `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`:

```js
const source = fs.readFileSync(__filename, 'utf-8');
expect(source).toContain('.hmrH(');
expect(source).not.toContain('module.hot');
expect(source).not.toContain('__webpack_module__.hot');
expect(import.meta.hot).toBe(import.meta.hot);
```

Keep the existing `typeof` assertions. In `parser-options.js`, retain the source-preservation assertion for `import.meta.hot` so the runtime requirement is not emitted when `hot: false`.

- [ ] **Step 2: Run the configCase and verify the current alias fails**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: FAIL because the generated source still contains `module.hot` and does not contain `.hmrH(`.

- [ ] **Step 3: Add the runtime global**

Insert `const WEBPACK_HOT_CONTEXT;` and `const HOT_CONTEXT;` immediately after `HMR_RUNTIME_STATE_PREFIX`, then add both property-name match arms immediately after the `HMR_RUNTIME_STATE_PREFIX` arm in `runtime_globals_property_name`:

```rust
/// Returns the webpack Hot object factory.
const WEBPACK_HOT_CONTEXT;

/// Returns the dedicated import.meta.hot context for a module id.
const HOT_CONTEXT;
```

```rust
RuntimeGlobals::WEBPACK_HOT_CONTEXT => "hmrW",
RuntimeGlobals::HOT_CONTEXT => "hmrH",
```

Do not add either context global to `INITIALIZE_OBJECT_GLOBALS`; `WEBPACK_HOT_CONTEXT` is defined by the existing webpack HMR runtime module and `HOT_CONTEXT` by the dedicated context runtime module.

- [ ] **Step 4: Add the presentational dependency**

Create `import_meta_hot_context.rs` with this public shape:

```rust
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, RuntimeGlobals, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};
use rspack_hash::{RspackHash, RspackHasher};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaHotDependency {
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl ImportMetaHotDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self { range, loc }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

impl RspackHash for ImportMetaHotDependency {
  fn hash(&self, state: &mut RspackHasher) {
    "ImportMetaHotDependency".hash(state);
    self.range.hash(state);
    RuntimeGlobals::HOT_CONTEXT.hash(state);
    RuntimeGlobals::MODULE_ID.hash(state);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaHotDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaHotDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaHotDependencyTemplate;

impl ImportMetaHotDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaHotDependency")
  }
}

impl DependencyTemplate for ImportMetaHotDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaHotDependency>()
      .expect("ImportMetaHotDependencyTemplate requires ImportMetaHotDependency");
    let module_argument = context.runtime_template.render_module_argument(
      context
        .compilation
        .get_module_graph()
        .module_by_identifier(&context.module.identifier())
        .expect("module graph module must exist")
        .get_module_argument(),
    );
    context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::HOT_CONTEXT | RuntimeGlobals::MODULE_ID);
    let getter = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HOT_CONTEXT);
    source.replace(
      dep.range.start,
      dep.range.end,
      format!("{getter}({module_argument}.id)"),
      None,
    );
  }
}
```

Export the module from `dependency/hmr/mod.rs` and register `ImportMetaHotDependencyTemplate` in `impl_plugin_for_js_plugin.rs`.

- [ ] **Step 5: Build and run the focused test**

Run:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected at this task boundary: compilation succeeds, but the test may still fail until Task 2 attaches the dependency from the parser. The build must not introduce Rust errors.

- [ ] **Step 6: Continue to Task 2 without committing**

Keep the failing test and scaffold changes in the worktree. Task 2 connects the parser, Task 3 adds dependency parsing, and Task 4 installs the runtime before the atomic slice is committed.

---

### Task 2: Split the Vite Parser Path from webpack HMR

**Files:**

- Modify: `crates/rspack_plugin_javascript/src/parser_plugin/hot_module_replacement_plugin.rs`
- Modify: `crates/rspack_plugin_javascript/src/visitors/dependency/util.rs`
- Modify: `crates/rspack_plugin_hmr/src/lib.rs`
- Test: `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`

**Interfaces:**

- Consumes: `ImportMetaHotDependency::new(range, loc)` from Task 1.
- Produces: `ImportMetaHotReplacementParserPlugin` for JS auto and ESM module types.
- Preserves: `ImportMetaWebpackHotReplacementParserPlugin` as webpack-only handling for `import.meta.webpackHot`.

- [ ] **Step 1: Add a parser regression that rejects alias-only webpack members**

Add to the configCase:

```js
expect(import.meta.hot.decline).toBeUndefined();
expect(import.meta.hot.status).toBeUndefined();
```

These properties belong to webpack `Hot`, not to the planned Vite-compatible context.

- [ ] **Step 2: Run the configCase and verify it fails on the current alias**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: FAIL because `import.meta.hot` still exposes the webpack hot object.

- [ ] **Step 3: Restore the existing parser plugin to webpack-only behavior**

Rename the existing parser to `ImportMetaWebpackHotReplacementParserPlugin`, then change its hooks back to exact `import.meta.webpackHot` matching:

```rust
if for_name == expr_name::IMPORT_META_WEBPACK_HOT
  && parser
    .javascript_options
    .import_meta()
    .is_known_property_enabled(ImportMetaKnownProperties::WEBPACK_HOT)
{
  // existing webpack evaluate/member/call behavior
}
```

Rename the old webpack constants to `IMPORT_META_WEBPACK_HOT`, `IMPORT_META_WEBPACK_HOT_ACCEPT`, and `IMPORT_META_WEBPACK_HOT_DECLINE`. Give `IMPORT_META_HOT` and `IMPORT_META_HOT_ACCEPT` to the dedicated surface, remove its `.decline` constant, and stop routing dedicated calls to the webpack handlers.

- [ ] **Step 4: Add the dedicated Vite parser plugin**

Add this plugin shape in the same file:

```rust
pub struct ImportMetaHotReplacementParserPlugin {
  _private: (),
}

impl ImportMetaHotReplacementParserPlugin {
  pub fn new() -> Self {
    Self { _private: () }
  }

  fn enabled(parser: &JavascriptParser) -> bool {
    parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::HOT)
  }

  fn add_context_dependency(parser: &mut JavascriptParser, span: Span) {
    parser.build_info.module_concatenation_bailout =
      Some(String::from("Vite-compatible import.meta.hot"));
    let range = DependencyRange::from(span);
    let loc = parser.to_dependency_location(range);
    parser.add_presentational_dependency(Box::new(
      ImportMetaHotDependency::new(range, loc),
    ));
  }
}
```

Implement `evaluate_identifier` for `IMPORT_META_HOT`, `member` for the base context, and leave `.accept` to Task 3's call handler. The member hook must call `add_context_dependency` instead of `create_hmr_expression_handler`.

- [ ] **Step 5: Register both parser plugins**

In `rspack_plugin_hmr/src/lib.rs`, register the webpack and Vite plugins independently:

```rust
if module_type.is_js_auto() {
  parser.add_parser_plugin(Box::new(ModuleHotReplacementParserPlugin::new()));
  parser.add_parser_plugin(Box::new(ImportMetaWebpackHotReplacementParserPlugin::new()));
  parser.add_parser_plugin(Box::new(ImportMetaHotReplacementParserPlugin::new()));
} else if module_type.is_js_esm() {
  parser.add_parser_plugin(Box::new(ImportMetaWebpackHotReplacementParserPlugin::new()));
  parser.add_parser_plugin(Box::new(ImportMetaHotReplacementParserPlugin::new()));
}
```

- [ ] **Step 6: Build and verify code generation**

Run:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: PASS. Generated code contains `.hmrH(` and does not contain `module.hot` for `import.meta.hot`; the `webpackHot` guard still compiles through `module.hot`.

- [ ] **Step 7: Continue to Task 3 without committing**

Keep the parser split in the same atomic worktree change; the runtime is intentionally installed in Task 4.

---

### Task 3: Resolve Vite Dependency-Accept Requests Without webpack Callback Wrapping

**Files:**

- Modify: `crates/rspack_core/src/dependency/dependency_type.rs`
- Rename: `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_accept.rs` to `import_meta_webpack_hot_accept.rs` and rename its Rust types with `Webpack`.
- Create: `crates/rspack_plugin_javascript/src/dependency/hmr/import_meta_hot_accept.rs`
- Modify: `crates/rspack_plugin_javascript/src/dependency/hmr/mod.rs`
- Modify: `crates/rspack_plugin_javascript/src/parser_plugin/hot_module_replacement_plugin.rs`
- Modify: `crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs`
- Modify: `crates/rspack_plugin_hmr/src/lib.rs`
- Test: `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`

**Interfaces:**

- Produces: `DependencyType::ImportMetaHotAccept`.
- Produces: `ImportMetaHotAcceptDependency::new(request, range)`; weak ESM dependency whose template replaces the request string with a module ID.
- Produces: `create_import_meta_accept_handler`, which never creates `ESMAcceptDependency`.

- [ ] **Step 1: Add code-generation assertions for all dependency forms**

Extend the configCase source:

```js
if (import.meta.hot) {
  import.meta.hot.accept('./dep', (mod) => mod);
  import.meta.hot.accept(['./dep'], (mods) => mods);
  import.meta.hot.accept(() => {});
  import.meta.hot.accept();
}
```

Assert the emitted source does not contain `__rspack_hmr_outdated`, because that identifier belongs to webpack's `ESMAcceptDependency` wrapper:

```js
expect(source).not.toContain('__rspack_hmr_outdated');
```

- [ ] **Step 2: Run the configCase and verify the dependency form fails**

Run the filtered configCase. Expected: FAIL because alias `.accept` is not yet transformed into weak dependencies and/or still emits the webpack wrapper.

- [ ] **Step 3: Add the Vite dependency type and implementation**

First rename the existing webpack dependency kind and implementation from `ImportMetaHotAccept`/`ImportMetaHotAcceptDependency` to `ImportMetaWebpackHotAccept`/`ImportMetaWebpackHotAcceptDependency`, preserving the rendered string `"import.meta.webpackHot.accept"` and behavior. Rename its source file to `import_meta_webpack_hot_accept.rs`. Then add the new neutral `ImportMetaHotAccept` kind rendered as `"import.meta.hot.accept"` and create `import_meta_hot_accept.rs` by following the renamed webpack implementation, with these deliberate differences:

```rust
#[cacheable_dyn]
impl Dependency for ImportMetaHotAcceptDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaHotAccept
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ImportMetaHotAcceptDependency {
  fn request(&self) -> &str { &self.request }
  fn user_request(&self) -> &str { &self.request }
  fn weak(&self) -> bool { true }
  fn factorize_info(&self) -> &FactorizeInfo { &self.factorize_info }
  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo { &mut self.factorize_info }
}
```

The template must replace only the literal request range with `runtime_template.module_id(...)`; it must not wrap callbacks or insert imports.

- [ ] **Step 4: Add the Vite-specific accept parser**

Implement a handler separate from `create_accept_handler`:

```rust
fn create_import_meta_accept_handler(
  &mut self,
  call_expr: &CallExpr,
) -> Option<bool> {
  self.build_info.module_concatenation_bailout =
    Some(String::from("Vite-compatible import.meta.hot.accept"));

  let dependencies = extract_deps(self, call_expr, |request, range| {
    Box::new(ImportMetaHotAcceptDependency::new(request, range))
  });
  self.add_dependencies(dependencies);
  self.walk_expr_or_spread(&call_expr.args);
  Some(true)
}
```

In `ImportMetaHotReplacementParserPlugin::call`, match only `IMPORT_META_HOT_ACCEPT`, add the base `ImportMetaHotDependency` for the callee object range, then invoke `create_import_meta_accept_handler`.

If the first argument evaluates to an array containing any non-string item, add an actionable parser warning naming `import.meta.hot.accept` and require a string literal or literal string array. Do not silently keep only the string elements.

- [ ] **Step 5: Register the dependency template and factory**

Register `ImportMetaHotAcceptDependencyTemplate` in the JavaScript plugin and map `DependencyType::ImportMetaHotAccept` to `normal_module_factory` in `HotModuleReplacementPlugin::compilation`.

- [ ] **Step 6: Build and run focused parser tests**

Run:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
```

Expected: PASS; dependency literals are module IDs, callback source is preserved, and no webpack outdated-ID wrapper is emitted.

- [ ] **Step 7: Continue to Task 4 without committing**

Keep the dependency parsing changes in the same atomic worktree change so no commit contains an undefined `hmrH` runtime.

---

### Task 4: Add the Dedicated Vite HotContext Runtime Module

- Modify: `crates/rspack_plugin_hmr/src/runtime/hot_module_replacement.ejs`

**Files:**

- Create: `crates/rspack_plugin_hmr/src/hot_context.rs`
- Create: `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs`
- Modify: `crates/rspack_plugin_hmr/src/lib.rs`
- Test: `tests/rspack-test/configCases/parsing/import-meta-hot/index.js`

**Interfaces:**

- Produces: `<require>.hmrH(moduleId) -> ImportMetaHotContext`.
- Produces: `<require>.hmrH.get(moduleId) -> ImportMetaHotState | undefined` for the apply runtime.
- Produces: `<require>.hmrH.dispose(moduleId, removed) -> void` for lifecycle cleanup.
- `ImportMetaHotState` fields: `data`, `selfAccepted`, `selfCallback`, `acceptCallbacks`, and `disposeCallbacks`.

- [ ] **Step 1: Add runtime-object assertions**

Add to the configCase:

```js
expect(import.meta.hot).toBe(import.meta.hot);
expect(import.meta.hot.data).toEqual({});
expect(typeof import.meta.hot.accept).toBe('function');
expect(typeof import.meta.hot.dispose).toBe('function');
```

Run the configCase. Expected: FAIL because `hmrH` is required but no runtime module defines it.

- [ ] **Step 2: Name the webpack factory explicitly**

In `runtime/hot_module_replacement.ejs`, keep the existing factory behavior unchanged but define it as `<%- define(WEBPACK_HOT_CONTEXT) %> = function (moduleId, me) { ... }` and change the interceptor assignment to `module.hot = <%- WEBPACK_HOT_CONTEXT %>(options.id, module)`. No dedicated-context state may be stored on or read from this webpack object.

- [ ] **Step 3: Create the runtime module wrapper**

Implement `hot_context.rs` using the same `RuntimeModule` pattern as `hot_module_replacement.rs`:

```rust
static HOT_CONTEXT_TEMPLATE: &str = include_str!("runtime/hot_context.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotContextRuntimeModule {}

impl HotContextRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotContextRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), HOT_CONTEXT_TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id().as_str(), None)
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> RuntimeModuleRuntimeRequirements {
    extract_runtime_globals_from_ejs(HOT_CONTEXT_TEMPLATE)
  }
}
```

- [ ] **Step 4: Implement the per-module registry**

Create `hot_context.ejs` with this complete state machine:

```js
var hotContexts = Object.create(null);
var hotData = Object.create(null);

function createHotContext(moduleId) {
  var state = {
    moduleId: moduleId,
    data: hotData[moduleId] || (hotData[moduleId] = {}),
    selfAccepted: false,
    selfCallback: undefined,
    acceptCallbacks: [],
    disposeCallbacks: []
  };

  var context = {
    get data() {
      return state.data;
    },
    accept: function (deps, callback) {
      if (deps === undefined) {
        state.selfAccepted = true;
        return;
      }
      if (typeof deps === "function") {
        state.selfAccepted = true;
        state.selfCallback = deps;
        return;
      }
      if (typeof deps === "string") {
        state.acceptCallbacks.push({
          deps: [deps],
          callback: callback,
          single: true
        });
        return;
      }
      state.acceptCallbacks.push({
        deps: deps.slice(),
        callback: callback,
        single: false
      });
    },
    dispose: function (callback) {
      state.disposeCallbacks.push(callback);
    }
  };

  state.context = context;
  hotContexts[moduleId] = state;
  return context;
}

<%- HOT_CONTEXT %> = function (moduleId) {
  var state = hotContexts[moduleId];
  return state ? state.context : createHotContext(moduleId);
};

<%- HOT_CONTEXT %>.get = function (moduleId) {
  return hotContexts[moduleId];
};

<%- HOT_CONTEXT %>.dispose = function (moduleId, removed) {
  var state = hotContexts[moduleId];
  if (!state) return;
  for (var i = 0; i < state.disposeCallbacks.length; i++) {
    state.disposeCallbacks[i](state.data);
  }
  delete hotContexts[moduleId];
  if (removed) delete hotData[moduleId];
};
```

Dependency request literals have already been converted to module IDs by Task 3, so the runtime stores IDs without resolving source strings.

- [ ] **Step 5: Register the runtime only when requested**

Add a `CompilationRuntimeRequirementInTree` hook to `HotModuleReplacementPlugin`:

```rust
if runtime_requirements.contains(RuntimeGlobals::HOT_CONTEXT) {
  runtime_modules_to_add.push((
    *chunk_ukey,
    HotContextRuntimeModule::new(&compilation.runtime_template).boxed(),
  ));
}
Ok(None)
```

Tap this hook from `Plugin::apply`. Do not add the runtime unconditionally from `additional_tree_runtime_requirements`.

- [ ] **Step 6: Build and run the configCase**

Run the binding build and filtered configCase. Expected: PASS; repeated `import.meta.hot` access returns the same object and the generated runtime contains `hmrH` only in chunks that use the feature.

- [ ] **Step 6: Commit the runtime module**

```bash
git add crates/rspack_core/src/runtime_globals.rs \
  crates/rspack_core/src/dependency/dependency_type.rs \
  crates/rspack_plugin_javascript/src/dependency/hmr \
  crates/rspack_plugin_javascript/src/parser_plugin/hot_module_replacement_plugin.rs \
  crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs \
  crates/rspack_plugin_javascript/src/visitors/dependency/util.rs \
  crates/rspack_plugin_hmr/src/hot_context.rs \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  crates/rspack_plugin_hmr/src/lib.rs \
  tests/rspack-test/configCases/parsing/import-meta-hot
git commit -m "feat: add dedicated vite hot context runtime"
```

---

### Task 5: Integrate Vite Accept State into HMR Propagation and Apply

**Files:**

- Modify: `crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs`
- Modify: `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/index.js`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/dep.js`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/a.js`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-dependency/b.js`

**Interfaces:**

- Consumes: `HOT_CONTEXT.get(moduleId)` and `.dispose(moduleId, removed)` from Task 4.
- Produces: dependency callbacks invoked after updated factories are installed, with `require(depId)` namespace values.
- Preserves: webpack accept callback execution and error handlers unchanged.

- [ ] **Step 1: Add a failing single/multiple dependency hotCase**

Create `index.js`:

```js
import { value } from './dep';
import { value as aValue } from './a';
import { value as bValue } from './b';

let acceptedDep;
let acceptedArray;

if (import.meta.hot) {
  import.meta.hot.accept('./dep', (mod) => {
    acceptedDep = mod;
  });
  import.meta.hot.accept(['./a', './b'], (mods) => {
    acceptedArray = mods;
  });
}

it('passes updated namespaces to Vite accept callbacks', async () => {
  expect(value).toBe(1);
  expect(aValue).toBe('a1');
  expect(bValue).toBe('b1');

  await NEXT_HMR();

  expect(acceptedDep.value).toBe(2);
  expect(acceptedArray.map((mod) => mod && mod.value)).toEqual(['a2', 'b2']);
  expect(value).toBe(2);
  expect(aValue).toBe('a2');
  expect(bValue).toBe('b2');
});
```

Create update fixtures:

```js
// dep.js
export const value = 1;
---
export const value = 2;
```

```js
// a.js
export const value = "a1";
---
export const value = "a2";
```

```js
// b.js
export const value = "b1";
---
export const value = "b2";
```

- [ ] **Step 2: Run the hotCase and verify propagation is unaccepted**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "hotCases/vite/import-meta-hot-dependency"
```

Expected: FAIL because the existing apply runtime does not consult the Vite registry.

- [ ] **Step 3: Teach graph traversal about Vite boundaries**

In `getAffectedModuleEffects`, read state without creating it:

```js
var hotState = <%- weak(HOT_CONTEXT) %>
  ? <%- HOT_CONTEXT %>.get(moduleId)
  : undefined;
```

Treat `hotState.selfAccepted` like webpack `_selfAccepted`. For each parent, stop propagation when any Vite accept record contains the updated child ID:

```js
var parentHotState = <%- weak(HOT_CONTEXT) %>
  ? <%- HOT_CONTEXT %>.get(parentId)
  : undefined;
var hotAccepted = parentHotState && parentHotState.acceptCallbacks.some(
  function (record) {
    return record.deps.indexOf(moduleId) !== -1;
  }
);
if (parent.hot._acceptedDependencies[moduleId] || hotAccepted) {
  if (!outdatedDependencies[parentId]) outdatedDependencies[parentId] = [];
  addAllToSet(outdatedDependencies[parentId], [moduleId]);
  continue;
}
```

Add `RuntimeGlobals::HOT_CONTEXT` as a weak EJS runtime requirement so webpack-only chunks do not pull the new runtime.

- [ ] **Step 4: Invoke Vite dependency callbacks with namespaces**

After the existing webpack callbacks, select each Vite record whose dependency list intersects `moduleOutdatedDependencies`:

```js
var hotState = <%- weak(HOT_CONTEXT) %>
  ? <%- HOT_CONTEXT %>.get(outdatedModuleId)
  : undefined;
if (hotState) {
  for (var i = 0; i < hotState.acceptCallbacks.length; i++) {
    var record = hotState.acceptCallbacks[i];
    var changed = record.deps.some(function (depId) {
      return moduleOutdatedDependencies.indexOf(depId) !== -1;
    });
    if (!changed) continue;
    try {
      var namespaces = record.deps.map(function (depId) {
        try {
          return <%- REQUIRE %>(depId);
        } catch (_err) {
          return undefined;
        }
      });
      record.callback(record.single ? namespaces[0] : namespaces);
    } catch (err) {
      reportError(err);
    }
  }
}
```

Do not modify the existing webpack callback loop or its error-handler context.

- [ ] **Step 5: Preserve old callback state through disposal**

Snapshot Vite callback records before disposing a parent module. Call `HOT_CONTEXT.dispose(moduleId, removed)` only for modules in `outdatedModules`; accepted parent modules remain alive and retain their dependency callbacks.

- [ ] **Step 6: Build and run the dependency hotCase plus webpack regressions**

Run:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "hotCases/vite/import-meta-hot-dependency"
pnpm run test -t "hotCases/esm-dependency-import/import-meta-webpack-hot"
pnpm run test -t "hotCases/runtime/accept"
```

Expected: all PASS; Vite callbacks receive namespaces while webpack callbacks retain outdated-ID semantics.

- [ ] **Step 7: Commit dependency apply integration**

```bash
git add crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  tests/rspack-test/hotCases/vite/import-meta-hot-dependency
git commit -m "feat: apply vite dependency hot updates"
```

---

### Task 6: Implement Self-Accept, Persistent Data, and Dispose

**Files:**

- Modify: `crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs`
- Modify: `crates/rspack_plugin_hmr/src/runtime/hot_context.ejs`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-self/index.js`
- Create: `tests/rspack-test/hotCases/vite/import-meta-hot-self/self.js`

**Interfaces:**

- Consumes: `ImportMetaHotState.selfAccepted`, `selfCallback`, `data`, and `disposeCallbacks`.
- Produces: successful self-re-evaluation callback `(namespace) => void`.
- Produces: fresh callback registrations with persistent `data` after every update.

- [ ] **Step 1: Add the failing self/lifecycle hotCase**

Create `index.js`:

```js
import './self';

it('runs the Vite self callback and preserves hot data', async () => {
  expect(globalThis.__importMetaHotInitial).toEqual({ value: 1, count: 0 });
  await NEXT_HMR();
  expect(globalThis.__importMetaHotAccepted).toEqual({ value: 2, count: 1 });
});
```

Create `self.js`:

```js
export const value = 1;

if (import.meta.hot) {
  import.meta.hot.data.count ||= 0;
  globalThis.__importMetaHotInitial = {
    value,
    count: import.meta.hot.data.count
  };
  import.meta.hot.dispose(data => {
    data.count += 1;
  });
  import.meta.hot.accept(mod => {
    globalThis.__importMetaHotAccepted = {
      value: mod.value,
      count: import.meta.hot.data.count
    };
  });
}

---

export const value = 2;

if (import.meta.hot) {
  import.meta.hot.dispose(data => {
    data.count += 1;
  });
  import.meta.hot.accept();
}
```

- [ ] **Step 2: Run the hotCase and verify the callback is not invoked**

Run the filtered hotCase. Expected: FAIL because the apply runtime does not yet re-require Vite self-accepted modules or call the old success callback.

- [ ] **Step 3: Snapshot self-accepted Vite modules**

When building `outdatedSelfAcceptedModules`, add an entry for Vite state independently of webpack state:

```js
if (hotState && hotState.selfAccepted) {
  outdatedSelfAcceptedModules.push({
    module: outdatedModuleId,
    type: 'import-meta-hot',
    callback: hotState.selfCallback,
  });
}
```

Keep the existing webpack entry shape and error-handler behavior unchanged.

- [ ] **Step 4: Dispose the old context but preserve its data**

During module disposal call:

```js
if (<%- weak(HOT_CONTEXT) %>) {
  var removed = appliedUpdate[moduleId] === warnUnexpectedRequire;
  <%- HOT_CONTEXT %>.dispose(moduleId, removed);
}
```

The Vite runtime deletes only the old context/callback registrations for an update and keeps `hotData[moduleId]`. It deletes both context and data when the module is removed.

- [ ] **Step 5: Re-evaluate and invoke the old self callback**

Split the self-accepted apply loop by entry type:

```js
if (item.type === "import-meta-hot") {
  try {
    var namespace = <%- REQUIRE %>(moduleId);
    if (typeof item.callback === "function") item.callback(namespace);
  } catch (err) {
    if (typeof item.callback === "function") item.callback(undefined);
    reportError(err);
  }
  continue;
}
```

The webpack branch must remain byte-for-byte equivalent except for surrounding dispatch structure.

- [ ] **Step 6: Build and run lifecycle and webpack regression cases**

Run:

```bash
pnpm run build:binding:dev
cd tests/rspack-test
pnpm run test -t "hotCases/vite/import-meta-hot-self"
pnpm run test -t "hotCases/runtime/self-accept-and-dispose"
pnpm run test -t "hotCases/recover/recover-after-self-error"
```

Expected: all PASS. Vite success callbacks receive the new namespace; webpack self callbacks remain error handlers.

- [ ] **Step 7: Commit self-accept lifecycle support**

```bash
git add crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs \
  crates/rspack_plugin_hmr/src/runtime/hot_context.ejs \
  tests/rspack-test/hotCases/vite/import-meta-hot-self
git commit -m "feat: support vite self accept lifecycle"
```

---

### Task 7: Publish the Opt-In Hot-Context Types

**Files:**

- Modify: `packages/rspack/module.d.ts`
- Create: `packages/rspack/import-meta-hot.d.ts`
- Modify: `packages/rspack/package.json`
- Modify: `tests/type-tests/resolution-bundler/index.ts`
- Verify locally without committing: `.superpowers/sdd/vite-hot-context-compat/`

**Interfaces:**

- Produces: `Rspack.ModuleNamespace`.
- Produces: `Rspack.ImportMetaHotContext` with only `data`, four `accept` overloads, and `dispose`.
- Produces: opt-in `@rspack/core/import-meta-hot` ambient declaration.
- Preserves: `Rspack.Hot` for webpack surfaces.

- [ ] **Step 1: Write failing type usages**

Add this type reference and the usages to `resolution-bundler/index.ts`:

```ts
/// <reference types="@rspack/core/import-meta-hot" />

if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.accept((mod) => {
    mod?.default;
  });
  import.meta.hot.accept('./dep', (mod) => {
    mod?.default;
  });
  import.meta.hot.accept(['./a', './b'] as const, (mods) => {
    mods[0]?.default;
    mods[1]?.default;
  });
  import.meta.hot.dispose((data) => {
    data.disposed = true;
  });

  // @ts-expect-error webpack-only API
  import.meta.hot.decline();
  // @ts-expect-error Vite transport API is not implemented by this runtime
  import.meta.hot.send('event');
}
```

- [ ] **Step 2: Run type tests and verify the old `Rspack.Hot` declaration fails expectations**

Run:

```bash
pnpm run test:type
```

Expected: FAIL because the Vite callback overloads are absent and webpack-only members are currently exposed.

- [ ] **Step 3: Define the Vite-shaped namespace type**

In `module.d.ts`, remove `hot?: Rspack.Hot` from global `ImportMeta` and add:

```ts
declare namespace Rspack {
  type ModuleNamespace = Record<string, any> & {
    [Symbol.toStringTag]: 'Module';
  };

  interface ImportMetaHotContext {
    readonly data: any;
    accept(): void;
    accept(cb: (mod: ModuleNamespace | undefined) => void): void;
    accept(dep: string, cb: (mod: ModuleNamespace | undefined) => void): void;
    accept(
      deps: readonly string[],
      cb: (mods: Array<ModuleNamespace | undefined>) => void,
    ): void;
    dispose(cb: (data: any) => void): void;
  }
}
```

Do not add `acceptExports`, `prune`, `on`, `off`, or `send` until their runtime semantics exist.

- [ ] **Step 4: Add the opt-in ambient entry**

Create `packages/rspack/import-meta-hot.d.ts`:

```ts
/// <reference path="./module.d.ts" />

interface ImportMeta {
  readonly hot?: Rspack.ImportMetaHotContext;
}
```

Add to the existing `exports` and `files` collections in `packages/rspack/package.json`:

```json
{
  "exports": {
    "./import-meta-hot": {
      "types": "./import-meta-hot.d.ts"
    }
  },
  "files": ["import-meta-hot.d.ts"]
}
```

Keep every existing export and published file entry.

- [ ] **Step 5: Verify the supported subset in a separate local package**

Keep `tests/type-tests/resolution-bundler/tsconfig.json` scoped to its own `index.ts`, so the committed Rspack opt-in declaration is verified in an isolated TypeScript program.

Create `.superpowers/sdd/vite-hot-context-compat/` locally, pin Vite 8.1.5, and import `ViteHotContext` directly from the Vite hot type module rather than loading `vite/client`. With `skipLibCheck: false`, verify structural assignability in both directions between `Rspack.ImportMetaHotContext` and `Pick<ViteHotContext, "data" | "accept" | "dispose">`.

Do not commit this local package, a Vite workspace dependency, a workspace lockfile change, or a `vite/client` fixture. Loading the complete ambient declarations also exposes the pre-existing unrelated `ImportMeta.glob` property merge conflict; do not change `import.meta.glob` as part of this work.

- [ ] **Step 6: Run type and package checks**

Run:

```bash
pnpm run test:type
pnpm run check-dependency-version
```

Expected: the committed Rspack opt-in type program passes, and the separate local package passes with `skipLibCheck: false` in both assignability directions.

- [ ] **Step 7: Commit the type surface**

```bash
git add packages/rspack/module.d.ts \
  packages/rspack/import-meta-hot.d.ts \
  packages/rspack/package.json \
  tests/type-tests/resolution-bundler/index.ts
git commit -m "feat: add import meta hot types"
```

---

### Task 8: Update Documentation and Run Final Verification

**Files:**

- Modify: `website/docs/en/api/runtime-api/hmr.mdx`
- Modify: `website/docs/zh/api/runtime-api/hmr.mdx`
- Verify: all files changed by Tasks 1–7

**Interfaces:**

- Documents: separate webpack and Vite HMR surfaces.
- Documents: `@rspack/core/import-meta-hot` for Rspack-only TypeScript projects and the runtime's supported subset for migrated projects with an existing HotContext declaration.

- [ ] **Step 1: Replace alias wording in English docs**

Document these contracts explicitly:

```md
Rspack exposes two HMR API families:

- `module.hot` and `import.meta.webpackHot` implement the webpack HMR API.
- `import.meta.hot` implements Rspack's Vite-compatible `accept`, `data`, and
  `dispose` runtime. It is a separate per-module context and is not an alias of
  `module.hot`.
```

Include examples for `accept(cb)`, `accept(dep, cb)`, and `accept(deps, cb)`, showing namespace callback parameters. State that `acceptExports`, `prune`, and custom events are not supported by this release.

- [ ] **Step 2: Mirror the contract in Chinese docs**

Use the same method list, limitations, type-entry instructions, and examples. Do not translate code identifiers.

- [ ] **Step 3: Format changed files**

Run:

```bash
pnpm run format:rs
pnpm run format:js
```

Expected: formatting completes without errors and only intended files change.

- [ ] **Step 4: Build before test execution**

Run:

```bash
pnpm run build:cli:dev
```

Expected: Rust binding and JavaScript packages build successfully.

- [ ] **Step 5: Run focused feature and regression tests**

Run:

```bash
cd tests/rspack-test
pnpm run test -t "configCases/parsing/import-meta-hot"
pnpm run test -t "hotCases/vite/import-meta-hot"
pnpm run test -t "hotCases/esm-dependency-import/import-meta-webpack-hot"
pnpm run test -t "hotCases/runtime/accept"
pnpm run test -t "hotCases/runtime/self-accept-and-dispose"
cd ../..
pnpm run test:type
```

Expected: all focused tests PASS.

- [ ] **Step 6: Run repository-required validation**

Run:

```bash
pnpm run test:rs
pnpm run test:unit
pnpm run lint:rs
cargo lint
cargo fmt --all --check
```

Expected: all commands PASS. Do not run storage or native-watcher tests that are known to hang in the sandbox.

- [ ] **Step 7: Inspect the final diff for accidental webpack changes**

Run:

```bash
git diff --check
git diff --stat
git diff -- crates/rspack_plugin_hmr/src/runtime/hot_module_replacement.ejs \
  crates/rspack_plugin_runtime/src/runtime_module/runtime/javascript_hot_module_replacement.ejs \
  packages/rspack/module.d.ts
```

Expected: no whitespace errors; webpack hot registration and error-handler behavior remain unchanged except for additive Vite-state checks in the apply runtime.

- [ ] **Step 8: Commit docs and verification adjustments**

```bash
git add website/docs/en/api/runtime-api/hmr.mdx \
  website/docs/zh/api/runtime-api/hmr.mdx
git commit -m "docs: document vite-compatible import meta hot runtime"
```

---

## Review Checkpoints

Request a focused code review after:

1. Task 3 — parser/dependency split is complete and no webpack wrapper is emitted.
2. Task 6 — runtime propagation, dependency namespaces, and self-accept lifecycle all pass.
3. Task 7 — the Rspack opt-in type program and the local hot-context subset compatibility package pass separately with `skipLibCheck: false`.
4. Task 8 — final verification completes.

At every checkpoint, reject changes that implement `import.meta.hot` by returning or mutating `module.hot`; the only allowed integration point is the HMR apply runtime reading the independent Vite context registry.
