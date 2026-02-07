# Module Federation In Rspack: Deep Technical Map

Last verified against repository state on 2026-02-07.

This document explains how Module Federation (MF) works in Rspack end-to-end, including:

- JS API layer and option normalization.
- JS -> NAPI -> Rust plugin bridge.
- Rust plugin graph, module factories, and runtime modules.
- Runtime globals/data contracts emitted into bundles.
- Enhanced (MF 1.5 runtime-tools) vs non-enhanced code paths.
- Tree-shaking shared fallback pipeline.
- Manifest generation pipeline.
- Test coverage map for critical behavior.

Branch update reflected in this revision (`feat/mf-layers`):

- `shareScope` now flows as `string | string[]` through JS options, NAPI bindings, and Rust MF internals.
- Layer-aware sharing fields (`request`, `issuerLayer`, `layer`) are now preserved through JS normalization and Rust consume/provide pipelines.
- Enhanced runtime path is array-scope aware; legacy non-enhanced runtime keeps scalar share-scope behavior.
- `container-1-5/*-layers-full` scenarios are now part of the documented verification surface.

## 1) Mental Model

Rspack MF has two stacked layers:

1. A compatibility/orchestration layer in `packages/rspack/src`.
2. Core implementation in `crates/rspack_plugin_mf`.

For MF v1.5 (`ModuleFederationPlugin`):

- JS layer resolves runtime package paths, generates an `entryRuntime` bootstrap module payload, and applies Rust-side runtime plugin(s).
- JS layer then applies `ModuleFederationPluginV1` with `enhanced: true`.
- V1 wiring applies container/reference/share plugins that are backed by Rust implementations.
- Rust plugins compile expose/remote/share modules and inject runtime modules.
- Runtime code sets `__webpack_require__.federation` and wires handlers (`remotes`, `consumes`, sharing init).

For MF v1.0 (`ModuleFederationPluginV1` directly):

- Same container/reference/share foundation, but no v1.5 runtime bootstrap orchestration.
- `enhanced` defaults to `false` unless explicitly enabled.

## 2) Touchpoint Inventory (By Layer)

### 2.1 Public JS API + Orchestration

- `packages/rspack/src/container/ModuleFederationPlugin.ts`
- `packages/rspack/src/container/ModuleFederationPluginV1.ts`
- `packages/rspack/src/container/ContainerPlugin.ts`
- `packages/rspack/src/container/ContainerReferencePlugin.ts`
- `packages/rspack/src/container/ModuleFederationRuntimePlugin.ts`
- `packages/rspack/src/container/ModuleFederationManifestPlugin.ts`
- `packages/rspack/src/container/options.ts`
- `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js`
- `packages/rspack/src/exports.ts`

### 2.2 JS Sharing/Tree-Shaking Orchestration

- `packages/rspack/src/sharing/ShareRuntimePlugin.ts`
- `packages/rspack/src/sharing/SharePlugin.ts`
- `packages/rspack/src/sharing/ConsumeSharedPlugin.ts`
- `packages/rspack/src/sharing/ProvideSharedPlugin.ts`
- `packages/rspack/src/sharing/TreeShakingSharedPlugin.ts`
- `packages/rspack/src/sharing/IndependentSharedPlugin.ts`
- `packages/rspack/src/sharing/CollectSharedEntryPlugin.ts`
- `packages/rspack/src/sharing/SharedContainerPlugin.ts`
- `packages/rspack/src/sharing/SharedUsedExportsOptimizerPlugin.ts`
- `packages/rspack/src/sharing/utils.ts`

### 2.3 JS -> Rust Binding Layer

- `crates/rspack_binding_api/src/raw_options/raw_builtins/raw_mf.rs`
- `crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs`
- `crates/node_binding/napi-binding.d.ts`

### 2.4 Rust MF Core

- `crates/rspack_plugin_mf/src/lib.rs`
- Container subsystem: `crates/rspack_plugin_mf/src/container/*`
- Sharing subsystem: `crates/rspack_plugin_mf/src/sharing/*`
- Manifest subsystem: `crates/rspack_plugin_mf/src/manifest/*`

Container files (complete):

- `crates/rspack_plugin_mf/src/container/container_entry_dependency.rs`
- `crates/rspack_plugin_mf/src/container/container_entry_module.rs`
- `crates/rspack_plugin_mf/src/container/container_entry_module_factory.rs`
- `crates/rspack_plugin_mf/src/container/container_exposed_dependency.rs`
- `crates/rspack_plugin_mf/src/container/container_plugin.rs`
- `crates/rspack_plugin_mf/src/container/container_reference_plugin.rs`
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_async.ejs`
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs`
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs`
- `crates/rspack_plugin_mf/src/container/embed_federation_runtime_sync.ejs`
- `crates/rspack_plugin_mf/src/container/expose_runtime_module.rs`
- `crates/rspack_plugin_mf/src/container/fallback_dependency.rs`
- `crates/rspack_plugin_mf/src/container/fallback_item_dependency.rs`
- `crates/rspack_plugin_mf/src/container/fallback_module.rs`
- `crates/rspack_plugin_mf/src/container/fallback_module_factory.rs`
- `crates/rspack_plugin_mf/src/container/federation_data_runtime_module.rs`
- `crates/rspack_plugin_mf/src/container/federation_modules_plugin.rs`
- `crates/rspack_plugin_mf/src/container/federation_runtime_dependency.rs`
- `crates/rspack_plugin_mf/src/container/hoist_container_references_plugin.rs`
- `crates/rspack_plugin_mf/src/container/mod.rs`
- `crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs`
- `crates/rspack_plugin_mf/src/container/remote_module.rs`
- `crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`
- `crates/rspack_plugin_mf/src/container/remote_to_external_dependency.rs`
- `crates/rspack_plugin_mf/src/container/remotesLoading.ejs`

Sharing files (complete):

- `crates/rspack_plugin_mf/src/sharing/collect_shared_entry_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/consume_shared_fallback_dependency.rs`
- `crates/rspack_plugin_mf/src/sharing/consume_shared_module.rs`
- `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs`
- `crates/rspack_plugin_mf/src/sharing/consumesCommon.ejs`
- `crates/rspack_plugin_mf/src/sharing/consumesInitial.ejs`
- `crates/rspack_plugin_mf/src/sharing/consumesLoading.ejs`
- `crates/rspack_plugin_mf/src/sharing/initializeSharing.ejs`
- `crates/rspack_plugin_mf/src/sharing/mod.rs`
- `crates/rspack_plugin_mf/src/sharing/provide_for_shared_dependency.rs`
- `crates/rspack_plugin_mf/src/sharing/provide_shared_dependency.rs`
- `crates/rspack_plugin_mf/src/sharing/provide_shared_module.rs`
- `crates/rspack_plugin_mf/src/sharing/provide_shared_module_factory.rs`
- `crates/rspack_plugin_mf/src/sharing/provide_shared_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs`
- `crates/rspack_plugin_mf/src/sharing/share_runtime_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/shared_container_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/shared_container_runtime_module.rs`
- `crates/rspack_plugin_mf/src/sharing/shared_used_exports_optimizer_plugin.rs`
- `crates/rspack_plugin_mf/src/sharing/shared_used_exports_optimizer_runtime_module.rs`

Manifest files (complete):

- `crates/rspack_plugin_mf/src/manifest/asset.rs`
- `crates/rspack_plugin_mf/src/manifest/data.rs`
- `crates/rspack_plugin_mf/src/manifest/mod.rs`
- `crates/rspack_plugin_mf/src/manifest/options.rs`
- `crates/rspack_plugin_mf/src/manifest/utils.rs`

### 2.5 Build-time Packaging For Browser Runtime

- `packages/rspack/rslib.config.ts`
- `packages/rspack/rslib.browser.config.ts`

## 3) End-to-End Execution Flow

### 3.1 User config -> JS plugin orchestration

`new rspack.container.ModuleFederationPlugin(options)` in `ModuleFederationPlugin.ts` does:

1. Resolve MF runtime package paths (`@module-federation/runtime-tools`, `@module-federation/webpack-bundler-runtime`, `@module-federation/runtime`) or throw install guidance if runtime-tools is missing.
2. Inject resolve aliases for runtime packages.
3. Detect `shared.*.treeShaking` entries and optionally apply `TreeShakingSharedPlugin`.
4. Compute runtime experiments (`asyncStartup`).
5. On `beforeRun`/`watchRun` (once), build `entryRuntime` string payload and apply Rust `ModuleFederationRuntimePlugin` with that payload.
6. Apply `ModuleFederationPluginV1` with `enhanced: true` (core container/remotes/shared mechanics).
7. Optionally apply `ModuleFederationManifestPlugin` if `manifest` enabled.

### 3.2 `entryRuntime` payload generation

`getDefaultEntryRuntime(...)` in `ModuleFederationPlugin.ts` creates a `data:text/javascript` module request:

`@module-federation/runtime/rspack.js!=!data:text/javascript,<content>`

Generated content includes:

- Import of bundler runtime package.
- Imports for each `runtimePlugins` entry.
- Runtime plugin instantiation array (`plugin(params)`).
- Normalized remote info map (`getRemoteInfos`).
- Container name, share strategy, library type.
- Tree-shaking shared fallback map from independent-share build.
- Embedded default runtime body from `moduleFederationDefaultRuntime.js` (or pre-injected `MF_RUNTIME_CODE` in browser build).

### 3.3 V1 plugin wiring (still used by v1.5)

`ModuleFederationPluginV1.apply` applies, in `afterPlugins`:

- `ShareRuntimePlugin`
- `ContainerPlugin` (if `exposes`)
- `ContainerReferencePlugin` (if `remotes`)
- `SharePlugin` (if `shared`)

`ModuleFederationPlugin` v1.5 is therefore additive orchestration around this V1 core.

### 3.4 Builtin plugin registration + NAPI bridge

JS wrappers are `RspackBuiltinPlugin`s. Their `raw()` returns `{ name: BuiltinPluginName, options }`.

Rust switch in `raw_builtins/mod.rs` maps builtin names to Rust plugin constructors:

- `ContainerPlugin`
- `ContainerReferencePlugin`
- `ShareRuntimePlugin`
- `ProvideSharedPlugin`
- `ConsumeSharedPlugin`
- `CollectSharedEntryPlugin`
- `SharedContainerPlugin`
- `SharedUsedExportsOptimizerPlugin`
- `ModuleFederationRuntimePlugin`
- `ModuleFederationManifestPlugin`

Typed raw option conversion is implemented in `raw_mf.rs`.

### 3.5 Rust compilation/runtime injection

Rust plugins hook multiple phases:

- `compilation` setup: dependency factories.
- `make`/`finish_make`: add entries/includes.
- normal module factory `factorize`/`create_module`: turn requests into MF-specific modules.
- runtime requirement hooks: inject runtime modules.
- optimize hooks: hoisting and used-exports optimization.
- process assets: manifest/stats emission and post-processing.

### 3.6 Bundle runtime behavior

At runtime, the emitted modules + runtime modules coordinate via globals on `__webpack_require__`, especially:

- `__webpack_require__.federation`
- `__webpack_require__.initializeSharingData`
- `__webpack_require__.initializeExposesData`
- `__webpack_require__.remotesLoadingData`
- `__webpack_require__.consumesLoadingData`

## 4) JS Layer Details

### 4.1 `ModuleFederationPlugin` (v1.5 facade)

File: `packages/rspack/src/container/ModuleFederationPlugin.ts`

Key behaviors:

- Option type extends V1 options and adds:
  - `runtimePlugins`
  - `implementation`
  - `shareStrategy`
  - `manifest`
  - `injectTreeShakingUsedExports`
  - `treeShakingSharedDir`
  - `treeShakingSharedExcludePlugins`
  - `treeShakingSharedPlugins`
  - `experiments.asyncStartup`
- Resolves runtime implementation path. If unresolved and not browser build, throws explicit install error for `@module-federation/runtime-tools`.
- Builds normalized `remoteInfos` by parsing `remotes` plus remote type inference.
- Generates runtime bootstrap payload and passes to Rust `ModuleFederationRuntimePlugin`.
- Applies V1 plugin with `enhanced: true`.
- Applies manifest plugin when enabled.

### 4.2 `ModuleFederationPluginV1`

File: `packages/rspack/src/container/ModuleFederationPluginV1.ts`

Key behaviors:

- Default library: `{ type: 'var', name: options.name }`.
- Default remoteType: `options.remoteType` else library type else `'script'`.
- Ensures output library type is enabled.
- Applies core plugins after plugin setup.

### 4.3 Container/reference wrappers

`ContainerPlugin.ts`:

- Normalizes exposes.
- Applies `ShareRuntimePlugin`.
- Registers builtin `ContainerPlugin` with raw options.

`ContainerReferencePlugin.ts`:

- Normalizes remotes.
- Creates externals map keyed by `webpack/container/reference/<remote>[/fallback-i]`.
- Special-case for `remoteType === 'module' | 'module-import'` and relative external (`.`): uses `ExternalsPlugin('import', ...)` to avoid static ESM cycle (self-remote/runtimeChunk single case).
- Applies `ShareRuntimePlugin`.
- Registers builtin `ContainerReferencePlugin`.

### 4.4 Share wrappers

`SharePlugin.ts`:

- Normalizes `shared` into consume/provide views.
- Applies `ConsumeSharedPlugin` and `ProvideSharedPlugin`.

`ConsumeSharedPlugin.ts` and `ProvideSharedPlugin.ts`:

- Normalize and pass rich options to Rust.
- Both apply `ShareRuntimePlugin`.

`ShareRuntimePlugin.ts`:

- Singleton per compiler via WeakSet.

### 4.5 Tree-shaking shared orchestration (JS)

`TreeShakingSharedPlugin.ts`:

- Detects shared entries with `treeShaking` config.
- Applies `SharedUsedExportsOptimizerPlugin` (non-secondary pass).
- Applies `IndependentSharedPlugin` to run child compilations for fallback assets.

`IndependentSharedPlugin.ts`:

- Launches independent compilers:
  - pass 1: collect shared entries via `CollectSharedEntryPlugin`.
  - pass 2: build per-share fallback container via `SharedContainerPlugin`.
- Builds `buildAssets` map used as MF shared fallback metadata.
- Can inject fallback metadata into manifest/stats assets.

### 4.6 Manifest wrapper (JS side)

`ModuleFederationManifestPlugin.ts`:

- Normalizes manifest options and filenames.
- Builds `remoteAliasMap` from `getRemoteInfos` (single-entry remotes with `entry` + `name`).
- Infers expose/shared metadata from MF config.
- Builds `buildInfo` from package.json/environment.
- Registers builtin Rust manifest plugin options.

### 4.7 Browser packaging of runtime function

`rslib.browser.config.ts`:

- Reads `moduleFederationDefaultRuntime.js`, transpiles/minifies it, evaluates it in VM, extracts function body, injects as `MF_RUNTIME_CODE` constant.
- Removes `createRequire` usage for browser bundle compatibility.

`rslib.config.ts`:

- Also emits a transformed runtime artifact for Node package dist.

## 5) NAPI/Raw Option Bridge

### 5.1 Raw option structs

Defined in `crates/rspack_binding_api/src/raw_options/raw_builtins/raw_mf.rs`:

- `RawContainerPluginOptions`, `RawExposeOptions`
- `RawContainerReferencePluginOptions`, `RawRemoteOptions`
- `RawProvideOptions`
- `RawConsumeSharedPluginOptions`, `RawConsumeOptions`
- `RawCollectShareEntryPluginOptions`
- `RawSharedContainerPluginOptions`
- `RawSharedUsedExportsOptimizerPluginOptions`
- `RawModuleFederationRuntimePluginOptions` (+ experiments)
- `RawModuleFederationManifestPluginOptions`

These convert into Rust plugin option structs in `rspack_plugin_mf`.

Current bridge behavior (important for layered MF):

- Share scope fields use `RawShareScope = Either<String, Vec<String>>`.
- Bridge conversion normalizes scopes to `Vec<String>` and defaults to `["default"]` when empty.
- Raw consume/provide payloads now carry optional `request`, `issuer_layer`, and `layer`.

### 5.2 Builtin enum and switch

- Builtin names exposed in `crates/node_binding/napi-binding.d.ts` (`BuiltinPluginName`).
- Construction switch in `crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs` instantiates concrete Rust plugins.

## 6) Rust Core Internals

### 6.1 Container subsystem

#### 6.1.1 `ContainerPlugin`

File: `crates/rspack_plugin_mf/src/container/container_plugin.rs`

- `compilation` hook sets dependency factories:
  - `ContainerEntry` -> `ContainerEntryModuleFactory`
  - `ContainerExposed` -> normal module factory
- `make` hook creates `ContainerEntryDependency`, calls federation hook bus (`add_container_entry_dependency`), and adds entry.
- Runtime requirement hooks:
  - Adds `STARTUP_CHUNK_DEPENDENCIES` for entry chunk with expose modules and dependent chunks.
  - If `CURRENT_REMOTE_GET_SCOPE` required, ensures `HAS_OWN_PROPERTY`; in `enhanced` mode adds `ExposeRuntimeModule`.

#### 6.1.2 Container entry dependency/module/factory

- `ContainerEntryDependency` carries container metadata and expose list.
- `ContainerEntryModuleFactory` converts dependency into `ContainerEntryModule` (or share-container variant).
- `ContainerEntryModule`:
  - Build phase:
    - For normal container: creates async blocks per expose, plus static exports `get/init`.
    - For share-container entry: creates fallback dep and static exports.
  - Codegen phase:
    - Normal mode (non-enhanced): emits internal `moduleMap`, `get`, `init`, and `definePropertyGetters` wiring.
    - Non-enhanced container init keeps legacy scalar behavior (`first share scope` with fallback to `"default"`).
    - Enhanced mode: exports wrappers to `__webpack_require__.getContainer` and `__webpack_require__.initContainer`, and stores `CodeGenerationDataExpose` for runtime module consumption.
    - Enhanced expose data keeps `shareScope` as array-capable metadata.
    - Share-container mode: emits share-container-specific init/factory bootstrap.

#### 6.1.3 `ExposeRuntimeModule`

File: `container/expose_runtime_module.rs`

- Finds `CodeGenerationDataExpose` from expose modules in initial chunks.
- Emits:
  - `__webpack_require__.initializeExposesData = { moduleMap, shareScope }` where `shareScope` is array-capable in enhanced mode.
  - fallback guards for `__webpack_require__.getContainer` and `__webpack_require__.initContainer`

### 6.2 Remote/reference subsystem

#### 6.2.1 `ContainerReferencePlugin`

File: `container/container_reference_plugin.rs`

- `compilation` sets factories:
  - `RemoteToExternal` -> normal module
  - `RemoteToFallbackItem` -> normal module
  - `RemoteToFallback` -> `FallbackModuleFactory`
- `factorize` intercepts matching remote requests and creates `RemoteModule`.
- Runtime requirement hook adds `RemoteRuntimeModule` when `ENSURE_CHUNK_HANDLERS` is present.

#### 6.2.2 `RemoteModule`

File: `container/remote_module.rs`

- Represents a remote import request.
- Build phase:
  - Single external: creates `RemoteToExternalDependency`.
  - Multiple externals: creates `FallbackDependency`.
  - Publishes dependency to federation hook bus (`add_remote_dependency`).
- Codegen phase:
  - Adds empty `SourceType::Remote` source.
  - Adds `SourceType::ShareInit` data (`DataInitInfo::ExternalModuleId`) at stage 20 for sharing init ordering.
  - Emits one share-init item per configured scope (defaults to `default` when none is configured).

#### 6.2.3 `RemoteRuntimeModule`

File: `container/remote_runtime_module.rs`

- Builds:
  - `chunkMapping` (chunk -> remote module ids)
  - `moduleIdToRemoteDataMapping` (`shareScope`, `name`, `externalModuleId`, `remoteName`)
- Emits `__webpack_require__.remotesLoadingData = {...}`.
- Share scope serialization is mode-aware:
  - enhanced: emits `shareScope` as scalar-or-array payload for runtime-tools compatibility.
  - non-enhanced: emits single scalar scope for legacy `remotesLoading.ejs` contract.
- Enhanced mode: emits stub for `__webpack_require__.f.remotes` (expected to be provided by bundler runtime).
- Non-enhanced: emits full remotes loader implementation from `remotesLoading.ejs`.

#### 6.2.4 Fallback chain

- `FallbackDependency` -> `FallbackModuleFactory` -> `FallbackModule`.
- `FallbackModule` executes fallback external modules in order until one resolves.

### 6.3 Sharing subsystem

#### 6.3.1 `ShareRuntimePlugin` + `ShareRuntimeModule`

`share_runtime_plugin.rs`:

- Adds `ShareRuntimeModule` when `SHARE_SCOPE_MAP` runtime requirement exists.

`share_runtime_module.rs`:

- Scans referenced chunks for `SourceType::ShareInit` codegen data.
- Aggregates by `share_scope` and `init_stage`.
- Emits:
  - share scope map initialization (`__webpack_require__.S` runtime global)
  - `__webpack_require__.initializeSharingData = { scopeToSharingDataMapping, uniqueName }`
- Enhanced mode: stubs `__webpack_require__.I` if missing.
- Non-enhanced: emits classic sharing runtime from `initializeSharing.ejs`.

`DataInitInfo` stages used by MF internals:

- Stage 10: provided shares (`ProvideSharedInfo`).
- Stage 20: remote externals (`ExternalModuleId`).

#### 6.3.2 `ProvideSharedPlugin` + `ProvideSharedModule`

`provide_shared_plugin.rs`:

- Classifies provides into resolved/unresolved/prefix maps.
- Matching keys now incorporate layer when present (`(layer)request` lookup shape).
- Infers version from package metadata when missing.
- Adds include entries in `finish_make` as `ProvideSharedDependency`.

`provide_shared_module.rs`:

- Builds fallback dependency to actual shared module (`ProvideForSharedDependency`), eager or async.
- Keeps layered and unlayered identities distinct in module/resource identifiers.
- Emits `CodeGenerationDataShareInit` with `ProvideSharedInfo` at init stage 10, including optional `layer`.

#### 6.3.3 `ConsumeSharedPlugin` + `ConsumeSharedModule`

`consume_shared_plugin.rs`:

- Resolves consume configs into resolved/unresolved/prefix categories.
- Matching uses `request` identity (not only map key alias), and supports issuer-layer-qualified lookups.
- Layered matching prefers exact layer-qualified entry and falls back to unlayered match when appropriate.
- Infers `requiredVersion` from nearest package.json dependency fields when not configured.
- Intercepts requests in factorize/create-module hooks to produce `ConsumeSharedModule`.
- Adds `ConsumeSharedRuntimeModule` in additional tree runtime requirements.

`consume_shared_module.rs`:

- Optional fallback dependency (eager or async block).
- Fallback dependency carries optional layer via `Dependency::get_layer`.
- Emits `CodeGenerationDataConsumeShared` with share key/scope/version flags, optional `layer`, and fallback factory.

#### 6.3.4 `ConsumeSharedRuntimeModule`

File: `consume_shared_runtime_module.rs`

- Builds:
  - `chunkMapping` for consumes
  - `moduleIdToConsumeDataMapping`
  - `initialConsumes`
- Emits `__webpack_require__.consumesLoadingData = {...}`.
- Runtime payload now includes layer-aware fields (`layer`) and array-capable share-scope data.
- Enhanced mode: stubs `__webpack_require__.f.consumes` if needed.
- Non-enhanced mode: emits `consumesCommon.ejs`, `consumesInitial.ejs`, `consumesLoading.ejs`.

#### 6.3.5 Shared container (independent shared fallback)

`shared_container_plugin.rs` + `shared_container_runtime_module.rs`:

- Builds share-container entry (`DependencyType::ShareContainerEntry`).
- Emits minimal federation object init for share container runtime.

### 6.4 Tree-shaking shared support

#### 6.4.1 `CollectSharedEntryPlugin` (Rust)

File: `sharing/collect_shared_entry_plugin.rs`

- Scans `ConsumeSharedModule`s.
- Collects actual fallback request paths and inferred versions.
- Version inference strategy:
  - pnpm path regex first
  - fallback to `node_modules/<pkg>/package.json` version
- Emits `collect-shared-entries.json` by default.

#### 6.4.2 `SharedUsedExportsOptimizerPlugin` (Rust)

File: `sharing/shared_used_exports_optimizer_plugin.rs`

- Tracks referenced exports for configured shared keys via `dependency_referenced_exports`.
- Applies side-effect aware export usage shaping during `optimize_dependencies`.
- Optionally injects runtime module `SharedUsedExportsOptimizerRuntimeModule` to set `__webpack_require__.federation.usedExports`.
- Updates stats/manifest JSON assets with `usedExports` in `process_assets`.

### 6.5 Runtime orchestration subsystem

#### 6.5.1 `ModuleFederationRuntimePlugin`

File: `container/module_federation_runtime_plugin.rs`

- Adds `FederationDataRuntimeModule` in additional tree runtime requirements.
- On `finish_make`, includes `entry_runtime` module request via `FederationRuntimeDependency` and federation hook bus.
- On `finish_modules`, marks `ContainerEntryModule` async when `experiments.async_startup` is enabled.
- Applies:
  - `EmbedFederationRuntimePlugin`
  - `HoistContainerReferencesPlugin`

#### 6.5.2 `FederationDataRuntimeModule`

File: `container/federation_data_runtime_module.rs`

- Initializes `__webpack_require__.federation` if absent.
- Emits `chunkMatcher(chunkId)` and `rootOutputDir` metadata used by federation runtime loading.

#### 6.5.3 `EmbedFederationRuntimePlugin` + module

Plugin file: `container/embed_federation_runtime_plugin.rs`.
Module file: `container/embed_federation_runtime_module.rs`.
Templates: `embed_federation_runtime_sync.ejs`, `embed_federation_runtime_async.ejs`.

Behavior:

- Tracks federation runtime dependency IDs via hook collector.
- Adds startup runtime requirements on eligible chunks:
  - always `STARTUP`
  - when async startup: also `STARTUP_ENTRYPOINT`, `ENSURE_CHUNK_HANDLERS`
- Injects `EmbedFederationRuntimeModule` into runtime chunks.
- Patches startup render behavior for entry chunks delegating to runtime chunk (sync mode explicit startup call).
- Runtime module:
  - sync mode: wraps previous startup with one-time federation module executions.
  - async mode: creates `mfStartupBase`, `mfAsyncStartup`, wraps startup/entrypoint startup to await federation initialization.

#### 6.5.4 `HoistContainerReferencesPlugin`

File: `container/hoist_container_references_plugin.rs`

- Collects container-entry, federation-runtime, and remote dependency IDs from hook bus.
- During `optimize_chunks`:
  - resolves transitive initial references
  - hoists modules to runtime chunks
  - disconnects from non-runtime chunks
  - removes now-empty chunks and records incremental mutations

### 6.6 Federation hook bus

File: `container/federation_modules_plugin.rs`

Compilation-scoped hook registry:

- `add_container_entry_dependency`
- `add_federation_runtime_dependency`
- `add_remote_dependency`

Used for decoupled coordination among container/reference/runtime/hoist plugins.

## 7) Runtime Data Contracts And Globals

### 7.1 Core globals populated by Rust runtime modules

- `__webpack_require__.federation` (baseline object; chunkMatcher/rootOutputDir)
- `__webpack_require__.initializeSharingData`
- `__webpack_require__.initializeExposesData`
- `__webpack_require__.remotesLoadingData`
- `__webpack_require__.consumesLoadingData`

### 7.2 `moduleFederationDefaultRuntime.js` integration

The generated `entryRuntime` executes default runtime code which:

1. Copies bundler runtime APIs into `__webpack_require__.federation`.
2. Initializes federation fields:
   - `libraryType`
   - `sharedFallback`
   - `consumesLoadingModuleToHandlerMapping`
   - `initOptions` (`name`, `shareStrategy`, `shared`, `remotes`, `plugins`)
   - `bundlerRuntimeOptions.remotes` metadata
3. Overrides runtime hooks:
   - `__webpack_require__.S`
   - `__webpack_require__.f.remotes`
   - `__webpack_require__.f.consumes`
   - `__webpack_require__.I`
   - `__webpack_require__.initContainer`
   - `__webpack_require__.getContainer`
4. Creates federation runtime instance:
   - `__webpack_require__.federation.instance = bundlerRuntime.init(...)`
5. Installs eager initial consumes when present.

Additional enhanced-runtime compatibility logic:

- Normalizes incoming share scopes to array form (`toScopeArray`) and re-emits runtime shape as needed (`toRuntimeScope`).
- Expands remote entries with multi-scope config into per-scope runtime records for bundler-runtime compatibility.
- Preserves layer in consume handler share metadata (`shareInfo.shareConfig.layer`).
- Ensures sharing init for relevant scopes runs before consume handlers in enhanced mode for cases where fallback/load ordering would otherwise race.

Guard condition:

- Runs when `(__webpack_require__.initializeSharingData || __webpack_require__.initializeExposesData) && __webpack_require__.federation`.

## 8) Option-To-Implementation Mapping

### 8.1 `name`, `filename`, `library`, `runtime`

- JS: normalized in `ModuleFederationPluginV1`, `ContainerPlugin`.
- Rust: `ContainerPlugin` add_entry options.
- Runtime: affects container entry export wrapper and manifest metadata.

### 8.2 `exposes`

- JS: normalized to key -> `{ import[], name? }`.
- Rust: `ContainerEntryModule` async blocks per expose.
- Runtime:
  - non-enhanced: internal `moduleMap/get/init` emitted in container module.
  - enhanced: `ExposeRuntimeModule` exposes `initializeExposesData` for default runtime hooking.

### 8.3 `remotes`, `remoteType`

- JS: normalized, externals created as `webpack/container/reference/<key>[/fallback-i]`.
- JS special-case: ESM remoteType + relative external uses `externalsType: import` to avoid static cycle.
- Rust: requests factorized to `RemoteModule`; runtime mapping emitted by `RemoteRuntimeModule`.

### 8.4 `shared`, `shareScope`

- JS API surface accepts `shareScope?: string | string[]` for:
  - `ModuleFederationPluginV1Options`
  - `ContainerPluginOptions`
  - `ContainerReferencePluginOptions` / `RemotesConfig`
  - `SharePluginOptions`, `ConsumesConfig`, `ProvidesConfig`
- JS normalization now preserves layered sharing fields:
  - consume path: `request`, `issuerLayer`, `layer`
  - provide path: `request`, `layer`
- `ConsumeSharedPlugin` fallback import semantics use request-aware defaults:
  - `request = item.request || key`
  - `import = item.import === false ? undefined : item.import || request`
- Rust provide path: `ProvideSharedPlugin` -> `ProvideSharedModule` -> share-init stage 10.
- Rust consume path: `ConsumeSharedPlugin` -> `ConsumeSharedModule` -> `ConsumeSharedRuntimeModule`.
- Rust share runtime: `ShareRuntimeModule` aggregates share-init data and initializes share scope map.

### 8.5 `runtimePlugins`

- JS: converted to dynamic imports and `plugin(params)` invocations in `entryRuntime` payload.
- Runtime: appended into `__webpack_require__.federation.initOptions.plugins`.

### 8.6 `implementation`

- JS: overrides runtime-tools resolution root; bundler runtime/runtime packages resolved relative to it.

### 8.7 `shareStrategy`

- JS: serialized into `entryRuntime`.
- Runtime: assigned to `__webpack_require__.federation.initOptions.shareStrategy`.

### 8.8 `manifest`

- JS: options normalized (filenames/path/buildInfo/remoteAliasMap/exposes/shared).
- Rust: emits stats + manifest assets at process-assets stage 0.

### 8.9 `experiments.asyncStartup`

- JS: forwarded to Rust runtime plugin.
- Rust effects:
  - marks container entry modules async in `finish_modules`
  - adds startup entrypoint/chunk-handler runtime requirements
  - uses async embed runtime template (`mfAsyncStartup` path)

### 8.10 Tree-shaking options

- `shared.<pkg>.treeShaking`: enables tree-shaking shared behavior.
- `injectTreeShakingUsedExports`: controls runtime usedExports injection.
- `treeShakingSharedDir`: output dir for independent shared fallback assets.
- `treeShakingSharedPlugins`, `treeShakingSharedExcludePlugins`: plugin control for fallback build.

## 9) Enhanced vs Non-Enhanced Behavior

Enhanced mode (`enhanced: true`) is used by `ModuleFederationPlugin` v1.5 path.

Key differences:

- Container expose:
  - enhanced: runtime stubs + `initializeExposesData` and default runtime wiring.
  - non-enhanced: container module emits legacy local get/init implementation directly.
- Sharing runtime:
  - enhanced: may emit stubs expecting bundler runtime handlers.
  - non-enhanced: emits full local implementations from EJS templates.
- Remotes/consumes loading:
  - enhanced: stubs `__webpack_require__.f.remotes/consumes` and delegates to bundler runtime.
  - non-enhanced: emits full local loader implementations.
- Multi-scope handling:
  - enhanced: supports array-capable share scope payloads end-to-end.
  - non-enhanced: intentionally keeps legacy scalar contracts (single emitted scope for remotes loading and container init path).

## 10) Manifest Pipeline

### 10.1 JS side

`ModuleFederationManifestPlugin.ts`:

- Determines `manifestFileName` and `statsFileName`.
- Adds inferred `exposes` and `shared` from MF config if not explicitly provided.
- Computes `remoteAliasMap` from remote infos.
- Adds `buildInfo` (`buildVersion`, `buildName`, optional tree-shaking metadata).

### 10.2 Rust side

`manifest/mod.rs`:

- Collects entrypoint/chunk/module relations.
- Builds stats structures for:
  - `metaData`
  - `shared`
  - `exposes`
  - `remotes`
- Emits:
  - stats JSON
  - manifest JSON derived from stats

`disable_assets_analyze` path:

- Emits structural metadata but leaves asset lists empty and marks unknown usage where appropriate.

## 11) Test Coverage Map (Representative)

### 11.1 Async startup and runtime wrapping

- `tests/e2e/cases/module-federation/async-startup-self-remote/*`
- `tests/e2e/cases/module-federation/async-startup-self-remote-runtimechunk-single/*`
- `tests/rspack-test/serialCases/container-1-5/2-async-startup-sync-imports/*`
- `tests/rspack-test/serialCases/container-1-5/3-async-startup-multi-entry-node/*`
- `tests/rspack-test/serialCases/container-1-5/4-async-startup-runtime-chunk-single/*`
- `tests/rspack-test/serialCases/container-1-5/5-async-startup-partial-runtime/*`

### 11.2 Manifest generation

- `tests/rspack-test/configCases/container-1-5/manifest/*`
- `tests/rspack-test/configCases/container-1-5/manifest-disable-assets-analyze/*`
- `tests/rspack-test/configCases/container-1-5/manifest-file-name/*`
- `tests/rspack-test/configCases/container-1-5/manifest-entry-filter/*`

### 11.3 Runtime plugins and plugin params

- `tests/rspack-test/configCases/container-1-5/runtime-plugin-with-params/*`
- `tests/rspack-test/configCases/container-1-5/runtime-plugin-with-used-exports/*`
- `tests/rspack-test/configCases/container-1-5/federation-instance-in-runtime-plugin/*`

### 11.4 Share strategy and sharing data

- `tests/rspack-test/configCases/container-1-5/share-strategy/*`
- `tests/rspack-test/configCases/container-1-5/provide-sharing-extra-data/*`
- `tests/rspack-test/serialCases/container-1-5/module-federation-with-shareScope/*`

### 11.5 Tree-shaking shared

- `tests/rspack-test/configCases/container-1-5/tree-shaking-shared-infer-mode/*`
- `tests/rspack-test/configCases/container-1-5/tree-shaking-shared-server-mode/*`

### 11.6 Hoisting behavior

- `tests/rspack-test/serialCases/container/reference-hoisting/*`
- `tests/rspack-test/watchCases/chunks/mf-hoisting/*`
- `tests/rspack-test/watchCases/chunks/mf-hoisting-multi-entries/*`

### 11.7 Runtime chunk routing and matcher data

- `tests/rspack-test/serialCases/container-1-5/multiple-runtime-chunk/*`
- `tests/rspack-test/serialCases/container-1-5/chunk-matcher/*`

### 11.8 Layered federation and multi-scope coverage

- `tests/rspack-test/serialCases/container-1-5/1-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/2-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/3-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/4-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/5-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/6-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/7-layers-full/*`
- `tests/rspack-test/serialCases/container-1-5/8-layers-full/*`
- `tests/rspack-test/configCases/sharing/tree-shaking-shared/*`

## 12) Notable Nuances / Gotchas

1. Runtime tools dependency:
- `@module-federation/runtime-tools` is optional peer in package metadata, but MF v1.5 runtime path will throw at build time if it cannot resolve and no `implementation` override is provided.

2. Option naming mismatch in docs vs code:
- Code uses `treeShakingSharedDir` (`ModuleFederationPluginOptions`), while docs currently show a section named `treeShakingDir`.

3. Manifest docs vs current behavior:
- Current tests/code for `disableAssetsAnalyze` keep `shared` and `exposes` entries but with empty asset lists (not fully omitted structures).

4. Remote alias map derivation:
- JS normalization for manifest alias mapping only includes remotes where a single script-style remote target provides both `entry` and `name`.

5. Internal `ContainerPlugin` default library type:
- Direct internal `ContainerPlugin` wrapper defaults to `global` if used directly.
- MF public flow through `ModuleFederationPluginV1` defaults to `var`.

6. Enhanced/non-enhanced split is intentional:
- In enhanced mode many runtime functions are expected from bundler runtime and only guarded stubs are emitted by Rust runtime modules.

7. Scope-array compatibility is mode-sensitive:
- Enhanced runtime paths are array-capable for share scopes.
- Non-enhanced runtime paths intentionally remain scalar to preserve legacy templates/contracts.

## 13) Quick “Where To Debug What” Guide

- Remote resolution/factorization issues:
  - `packages/rspack/src/container/ContainerReferencePlugin.ts`
  - `crates/rspack_plugin_mf/src/container/container_reference_plugin.rs`
  - `crates/rspack_plugin_mf/src/container/remote_module.rs`

- Shared version/strict/singleton behavior:
  - `packages/rspack/src/sharing/SharePlugin.ts`
  - `crates/rspack_plugin_mf/src/sharing/provide_shared_plugin.rs`
  - `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
  - `crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs`

- Runtime startup sequencing (`mfAsyncStartup`, `__webpack_require__.x`):
  - `crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs`
  - `crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs`
  - `crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs`
  - `crates/rspack_plugin_mf/src/container/embed_federation_runtime_async.ejs`

- Manifest generation/content:
  - `packages/rspack/src/container/ModuleFederationManifestPlugin.ts`
  - `crates/rspack_plugin_mf/src/manifest/mod.rs`
  - `crates/rspack_plugin_mf/src/manifest/asset.rs`
  - `crates/rspack_plugin_mf/src/manifest/utils.rs`

- Tree-shaking shared fallback and used exports:
  - `packages/rspack/src/sharing/TreeShakingSharedPlugin.ts`
  - `packages/rspack/src/sharing/IndependentSharedPlugin.ts`
  - `crates/rspack_plugin_mf/src/sharing/collect_shared_entry_plugin.rs`
  - `crates/rspack_plugin_mf/src/sharing/shared_used_exports_optimizer_plugin.rs`

---

If you need to extend MF behavior, most high-impact extension points are:

- `runtimePlugins` (JS runtime-tools plugin system).
- Rust runtime modules (`RemoteRuntimeModule`, `ConsumeSharedRuntimeModule`, `ShareRuntimeModule`, `EmbedFederationRuntimeModule`).
- Federation hook bus (`FederationModulesPlugin`) for cross-plugin dependency signaling.
