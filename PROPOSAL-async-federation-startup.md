# Proposal: Native Async Startup Support for Module Federation in Rspack

## 1. Background and Motivation
- Module Federation runtimes increasingly assume an *async startup contract*: the host must delay executing entry modules until remote containers, shared scopes, and any startup hooks finish resolving.
- The `@module-federation/enhanced` project already exposes an `experiments.asyncStartup` toggle. When enabled it:
  - Forces webpack’s `StartupChunkDependenciesPlugin` into async mode even for sync loaders like `require`.
  - Replaces the generated startup bootstrap so it awaits `Promise.all` on `__webpack_require__.f.remotes` / `.__consumes` handlers before running entry modules.
  - Tracks async work via `promiseTrack` and integrates with `RuntimeGlobals.startupEntrypoint`.
- Rspack’s Module Federation implementation currently mirrors the synchronous webpack bootstrap:
  - `StartupChunkDependenciesPlugin` only emits async code when the **chunk loading type** itself is async (`async-node`, `import()`, `importScripts`). The default `require` loader stays synchronous.
  - `EmbedFederationRuntimePlugin` wraps `RuntimeGlobals::STARTUP` to execute federation runtime dependencies, but it forwards whatever `prevStartup()` returns without forcing a promise.
  - `RuntimeGlobals::ENSURE_CHUNK` already aggregates promises from `__webpack_require__.f.*` handlers (e.g. `remotes`, `consumes`), yet the surrounding startup path may ignore those promises.
- Result: when a Node/SSR app enables Module Federation, remote containers can begin loading asynchronously but the entry chunk continues executing immediately, violating the federation runtime contract.

## 2. Goals
- Provide first-class async startup behavior whenever federation requests it, regardless of chunk loading strategy.
- Align Rspack’s runtime semantics with the `@module-federation/enhanced` experiment so the same application code works on both bundlers.
- Preserve backwards compatibility: synchronous startup remains the default unless federation explicitly opts in.
- Keep the change localized (no breaking change to unrelated runtime paths) and covered by integration tests.

## 3. Current Runtime Analysis
### 3.1 Enhanced async startup experiment
- Hooks `ModuleFederationPlugin` to install a custom `StartupChunkDependenciesPlugin` wrapper that always sets `{ asyncChunkLoading: true }`.
- Generates a startup fragment that:
  - Pushes async operations into `promiseTrack`.
  - Calls `__webpack_require__.x()` (startup extensions) and both `__webpack_require__.f.remotes` and `.consumes`.
  - Wraps entry execution in `Promise.all(promiseTrack).then(...)` or `await` (TLA).

### 3.2 Rspack runtime today
- `StartupChunkDependenciesPlugin` ([`crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs`](crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs)) toggles async behavior via its `async_chunk_loading` flag; `ChunkLoadingType::Require` passes `false`.
- The factory that wires runtime plugins (`enable_chunk_loading_plugin` in [`crates/rspack_plugin_runtime/src/lib.rs`](crates/rspack_plugin_runtime/src/lib.rs)) always instantiates `StartupChunkDependenciesPlugin::new(..., false)` for `ChunkLoadingType::Require`, so Node/CommonJS builds never emit the async templates today.
- `StartupChunkDependenciesRuntimeModule` and `StartupEntrypointRuntimeModule` already contain *both* async and sync templates, chosen by that flag.
- `EmbedFederationRuntimePlugin` ([`crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs`](crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs)) injects the federation runtime and wraps `__webpack_require__.x`, but does not ensure async resolution of remote handlers.
- Its `render_startup` hook inserts a plain `__webpack_require__.x();` call for entry chunks that proxy to runtime chunks, so any promise returned from `STARTUP` is dropped on the floor.
- JS bootstrap (`crates/rspack_plugin_javascript/src/plugin/mod.rs`) assigns `var __webpack_exports__ = __webpack_require__.x();` and proceeds synchronously; consumers of `__webpack_exports__` may still work if the variable is a promise, but cross-runtime helpers (e.g. CommonJS runner) often expect the exports object immediately.
- No config plumbing exists for `experiments.asyncStartup`, so the binding cannot pass the requirement through.

## 4. Proposed Native Implementation
### 4.1 Async startup gate and detection
### 4.1 Async startup flag lifecycle
1. **JavaScript API** – primary path is to extend `ModuleFederationPluginOptions` with `experiments?: { asyncStartup?: boolean; ... }` and forward the boolean into `ModuleFederationRuntimePlugin` during `apply`.  
   - If pushing a new property onto the federation options is not feasible, fall back to a `mfAsyncStartup?: boolean` entry on the top-level `experiments` object. In that mode, `ModuleFederationPlugin` reads `compiler.options.experiments.mfAsyncStartup === true` and sets the runtime flag automatically so consumers have a universal switch.
2. **Bindings** – add `async_startup: Option<bool>` to `RawModuleFederationRuntimePluginOptions` and to `ModuleFederationRuntimePluginOptions` so Rust receives the signal. Regenerate the napi typings.
3. **Registry** – inside `ModuleFederationRuntimePlugin`, record the flag against the current `CompilationId` (e.g. using a `LazyLock<FxDashMap<CompilationId, bool>>`). When using the global `mfAsyncStartup` fallback, treat the federation flag as `federationFlag || globalFlag`. Register a small hook on `compilation.finish` (or similar) to remove the entry once we’re done.

> **Implementation gap surfaced during prototyping**
>
> The async flag alone is not sufficient—Remote runtime metadata is stored in `remotesLoadingData` and only consumed when `__webpack_require__.f.remotes` runs. Remote modules rarely appear in the normal “entry dependent chunk” graph, so simply flipping the async template still skips the remote loader. The startup path must explicitly fold the bundler runtime’s handlers into the startup promise chain (the way `@module-federation/enhanced`’s `generateEntryStartup` does with `ensureChunkHandlers.remotes/consumes`). Without that, `__webpack_require__(moduleId)` (e.g. `146` for `containerA/ComponentA`) executes before the remote stub is registered and we throw. Any final design must include:
>
> - Reducing over `__webpack_require__.f.remotes` / `.consumes` for the current chunk id so the bundler runtime patches `__webpack_require__.m[id]` before the entry runs. A plain `Promise.all([...RuntimeGlobals.ensureChunk...])` is not enough.
> - Guaranteeing `RemoteRuntimeModule` executes before the entry so those handlers exist (runtime-module ordering matters when async startup is enabled).

### 4.2 Chunk-level detection and caching
1. Augment `StartupChunkDependenciesPlugin` with a `requires_async_startup(compilation, chunk_ukey)` helper that:
   - Returns `true` immediately when the chunk loading strategy is already async (`Import`, `Jsonp`, `ImportScripts`, `AsyncNode`).
   - Otherwise checks the registry; if the project requested async startup, continue to federation heuristics:
     * runtime requirements include `INITIALIZE_SHARING`, `SHARE_SCOPE_MAP`, or `CURRENT_REMOTE_GET_SCOPE`;
     * the chunk contains modules emitted as `SourceType::Remote` or `SourceType::ConsumeShared`;
     * federation runtime modules (`webpack/runtime/remotes_loading`, `consumes_loading`, etc.) are present.
   - Even without the explicit flag, fall back to auto detection when the runtime requirements set indicates federation and `ENSURE_CHUNK_HANDLERS` is present (covers advanced cases such as custom remotes).
2. Cache the decision in an `FxHashMap<ChunkUkey, bool>` on the plugin so subsequent hooks reuse the computation.

### 4.3 Wiring runtime modules
1. Pass the cached boolean into both runtime modules:
   - `StartupChunkDependenciesRuntimeModule::new(is_async)`
   - `StartupEntrypointRuntimeModule::new(is_async)`
2. Inside `StartupChunkDependenciesRuntimeModule::generate`, double-check the helper (for safety) and emit the asynchronous `Promise.all` body when required—even for `ChunkLoadingType::Require`.
3. `StartupEntrypointRuntimeModule` already exposes async and sync templates; we simply select the async template whenever the helper returns `true`.

### 4.4 Bootstrap behaviour across output formats
- **CommonJS / target: node** – ensure `rspack_plugin_javascript` assigns the startup promise to `module.exports`, while still supporting synchronous consumers by resolving immediately when the promise settles synchronously.
- **Script / var libraries** – write the promise onto the global container (e.g. `self[name] = exportsPromise`) and keep compatibility by returning the resolved exports via `exportsPromise.then(...)`.
- **ESM (`ModuleChunkFormatPlugin`)** – generate `const __webpack_exports__Promise = __webpack_require__.x();` and re-export as `export default await __webpack_exports__Promise` (or `return exportsPromise.then(...)` when TLA is unavailable). Forward named exports via helpers that await the promise.
- **Async chunk loaders** – no behavioural change required; the startup promise already chains the promises returned by `__webpack_require__.e`.

### 4.5 Interaction with other runtime plugins
- `CommonJsChunkLoadingPlugin` still installs the synchronous loader for `ChunkLoadingType::Require`; the new startup promise simply resolves immediately when no async handlers are registered, preserving fast paths.
- `ModuleChunkFormatPlugin` must import the promise and wait for it before executing entry modules, mirroring the script/global bootstraps.
- `EmbedFederationRuntimeModule` continues to wrap `prevStartup` with `Promise.resolve`, ensuring additional runtime hooks (e.g. other plugins) can compose without being aware of the async transition.

### 4.6 Hook ordering
- `ModuleFederationRuntimePlugin` registers the flag during `CompilerCompilation`, before runtime plugins execute.
- Federation plugins (`ContainerPlugin`, `ContainerReferencePlugin`, `EmbedFederationRuntimePlugin`) add their runtime requirements in the same compilation phase, so by the time `StartupChunkDependenciesPlugin` evaluates a chunk, the requirements already contain federation markers.
- `ModuleChunkFormatPlugin` runs after runtime modules emit their code, allowing it to reuse the cached boolean to choose the appropriate bootstrap.

### 4.7 Testing and validation
1. **Unit tests**
   - Verify the helper returns `true` for combinations of runtime globals, source types, and explicit flags.
   - Confirm `StartupChunkDependenciesRuntimeModule` emits the expected async/sync templates.
   - Ensure `EmbedFederationRuntimeModule` still correctly wraps startup with a promise.
2. **Integration tests**
   - Node/CommonJS host with `experiments.asyncStartup = true`: `require("./dist/main.js")` should return a promise that resolves to the entry exports and waits for remote loading.
   - Browser/script host: confirm the global entry receives a promise and side-effects run only after awaiting it.
   - ESM output (`experiments.outputModule`): top-level `await` (or promise chain) should expose the federated exports after remotes resolve.
   - Negative control (async flag off, no remotes): startup remains synchronous.
3. **Parity check** – compare generated startup fragments and runtime requirements with the enhanced webpack plugin to ensure compatibility.
4. **Performance** – benchmark a representative federation build to confirm that the extra promise handling does not regress cold start measurably.

## 5. Compatibility Considerations
- Async startup remains opt-in at the federation plugin level; existing users see no change.
- Document that the entry bundle produces a promise when the flag is enabled, and suggest awaiting it in application code (Node/SSR or browser).
- Update CLI/help text and typings so the flag is discoverable.
- Highlight that enabling async startup aligns Rspack with the latest `@module-federation/runtime` expectations.

## 6. Open Questions
1. Should we add a global `experiments.forceAsyncStartup` for non-federation runtimes that still want the behaviour?
2. How do we handle legacy environments without native promises or top-level `await`? (Likely polyfill guidance.)
3. Do we auto-enable async startup when detecting federation even if the flag is false, or keep that as a warning-only path?
4. How should SSR frameworks detect and await the startup promise automatically?

## 7. Status & Remaining Work (October 16 2025)
- ✅ JS & Rust plumbing: `moduleFederation.experiments.asyncStartup` and the fallback `experiments.mfAsyncStartup` flow through napi into the core `Experiments` struct.
- ✅ Runtime detection: `StartupChunkDependenciesPlugin` now auto-selects the async startup templates whenever the flag is set or federation runtime globals/modules are present.
- ✅ Regression coverage: `tests/rspack-test/serialCases/container-1-5/async-startup-no-dynamic` exercises federation async startup without relying on dynamic imports.
- ✅ CommonJS bootstraps (`CommonJsChunkFormatPlugin`) now return a promise when async startup is enabled.
- ✅ Global script bootstraps (`ArrayPushCallbackChunkFormatPlugin`) wrap startup in a promise under the async flag.
- ✅ Module chunk (ESM) bootstraps export an awaited promise when async startup is enabled.
- ✅ Added smoke checks that emitted CommonJS, global script, and ESM bundles contain the promise-aware bootstrap.
- ❌ Need end-to-end runtime assertions (especially verifying resolved values) that hosts receive a promise.
- ⚠️ `pnpm run test:unit` currently fails due to existing Jest haste-map collisions and long-running watch tests; no new failures introduced by async-startup changes.
- ⚠️ `pnpm run lint:type` currently fails due to the repository’s warning budget (pre-existing issue); revisit once refactor lands.

Delivering the remaining items will make async startup a native capability of Rspack’s runtime while keeping behaviour strictly opt-in for federation users.
