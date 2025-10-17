# Proposal: Async Startup Support for Module Federation in Rspack

## 1. Motivation
- Modern Module Federation runtimes (e.g. `@module-federation/enhanced`) assume hosts defer executing entry modules until the federation runtime, remote containers, and shared scopes are ready.
- Today Rspack only emits async startup code when the chunk loading type is inherently async (e.g. `import()`), so Node/CommonJS hosts run entry modules immediately and break the async contract.
- Previous attempts bolted async behavior into runtime modules (`__webpack_require__.x` wrappers, ad‑hoc installers). That approach diverges from Webpack’s implementation, is hard to reason about, and caused regressions in the serial/stats suites.
- We want to align with the “enhanced” implementation: generate async-ready startup code at compile time, keep the runtime thin, and only activate the behavior when federation actually needs it.

## 2. Goals
1. Match Webpack + `@module-federation/enhanced` semantics: hosts that opt into `experiments.asyncStartup` await remote/consume handlers before executing entry code.
2. Keep synchronous startup untouched when federation async startup is disabled.
3. Avoid runtime monkey patches (no `__webpack_require__.federation.installRuntime`, no document shims).
4. Limit the blast radius: only entry chunks that require MF async startup should see the new code.
5. Preserve the existing CommonJS/ESM entry assertions (tests expect the generated files to contain the promise-based wrappers).

## 3. Reference Implementation (core/packages/enhanced)
- Adds `federation-entry-startup` runtime requirement when async startup is enabled.
- Uses `MfStartupChunkDependenciesPlugin` to hook `JavascriptModulesPlugin.renderStartup`:
  - Resolves runtime chunks and explicitly imports them (`import * as __webpack_chunk_X__ …`).
  - Emits code that defers to `RuntimeGlobals.startupEntrypoint`, wrapping it in `Promise.all`.
  - Injects the promise-based CommonJS export (`module.exports = Promise.resolve().then(…)`) / ESM default await.
- Shares the async logic across CommonJS and ESM formats so the emitted code is clean and test friendly.

## 4. Current Rspack Status (main branch)
- `StartupChunkDependenciesRuntimeModule` only toggles async behavior based on chunk loading type.
- `EmbedFederationRuntimeModule` simply wraps `RuntimeGlobals::STARTUP` and calls the embedded federation modules synchronously.
- CommonJS/ESM chunk format plugins conditionally wrap startup in `Promise.resolve().then`, but only when `mf_async_startup` is on; there’s no awareness of which chunks actually need it.
- No render-startup hook injects the promise chain, so the runtime still runs immediately.

## 5. Proposed Changes

### 5.1 Detect async startup via plugin hooks (not runtime hacks)
- Introduce a Rust analogue of `MfStartupChunkDependenciesPlugin` that:
  - Hooks `JavascriptModulesPlugin::render_startup`.
  - If the chunk has runtime + federation async startup enabled, replace the startup source with the enhanced-style wrapper.
  - Emits the CommonJS `Promise.resolve().then` bootstrap and the ESM async default export inside the hook instead of via runtime modules.

### 5.2 Guard by federation necessity
- Only apply the wrapper when:
  - `compiler.options.experiments.mfAsyncStartup` (or MF plugin `experiments.asyncStartup`) is true, **and**
  - The chunk either contains remote/consume modules or the MF runtime dependency.
- For plain chunks (stats cases) the startup code should remain unchanged.

### 5.3 Runtime adjustments
- Revert `StartupChunkDependenciesRuntimeModule` to its original synchronous form; it should again rely on the template’s original async logic (`Promise.all([...ensureChunk])`) when chunk loading is async.
- Remove the ad-hoc `__webpack_require__.federation.installRuntime` and environment shims from `EmbedFederationRuntimeModule`.
- Ensure the MF runtime dependency is executed before the entry promise chain by sequencing it at the start of the generated startup wrapper (see Section 5.1).

### 5.4 CommonJS / ESM output
- Move the `Promise.resolve().then(...)` emission into the render hook so the generated bundle includes the exact string the tests look for (e.g. `module.exports = Promise.resolve().then`).
- For ESM, generate `const __webpack_exports__Promise = Promise.resolve().then(async () => …)` and `export default await __webpack_exports__Promise;` mirroring the enhanced implementation.
- Drop the existing runtime-side injection in `ModuleChunkFormatPlugin` and `CommonJsChunkFormatPlugin` once the render hook handles it.

## 6. Implementation Plan
1. **Restore runtime to baseline**: ensure `startup_chunk_dependencies.rs` and `embed_federation_runtime_module.rs` match the last known-good revision (no runtime wrappers).
2. **Create `MfAsyncStartupPlugin` (Rust)** mirroring the TS plugin:
   - Collect whether the compilation has MF async startup enabled.
   - Hook `render_startup` and, when conditions are met, replace `RenderSource` with the promise-based wrapper (reuse logic from `core/packages/enhanced/src/lib/startup/MfStartupChunkDependenciesPlugin.ts` and `StartupHelpers.ts`).
3. **Wire the plugin** inside `ModuleFederationPlugin` (Rust) when `mf_async_startup` is true.
4. **Update CommonJS/ESM chunk format plugins** to remove the runtime-side wrapping and rely on the render hook.
5. **Retain existing JS-facing expectations** (tests that read `main.js`/`main.mjs`) by keeping the generated code textually identical to the enhanced output.
6. **Ensure stats cases stay stable**: only insert extra runtime modules when MF async startup actually applies (filter by chunk contents and requirements).
7. **Testing**:
   - Serial suite (`pnpm --filter @rspack/tests test:base -- Serial.test.js`).
   - Stats, defaults, and other snapshot suites affected previously.
   - CLI-generated artifacts to make sure CommonJS/ESM wrappers match expectations.

## 7. Open Questions
- How to best determine “chunk needs MF async startup” from Rust (mirror logic from the enhanced plugin using chunk graph + runtime requirements)?
- Do we need additional tests covering hosts with only consumes/no remotes to ensure the wrapper still fires?
- Should the MF plugin expose `experiments.asyncStartup` for granular per-host control (currently we rely on global `mfAsyncStartup`)?

## 8. Deliverables
- New async-startup render hook that mirrors `@module-federation/enhanced`.
- Clean runtime modules (no federation-specific hacks).
- Updated unit/integration tests ensuring the generated code matches the expected promise-based bootstrap.
- Documentation of the new behavior (this proposal and CHANGELOG entry).*** End Patch*** End Patch
