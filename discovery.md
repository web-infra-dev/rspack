# MF Layers Runtime Parity Investigation

## Scope

- Repo: `/Users/zackjackson/rspack`
- Branch: `feat/mf-layers`
- Question: why do layered serial cases regress after reverting the custom `__webpack_require__.f.consumes` wrapper, even though `core/packages/enhanced` delegates directly to bundler-runtime and its layer tests pass?
- Constraint: keep `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js` thin and aligned with bundler-runtime ownership. The remaining bug should be found in emitted data, plugin state, or test/config parity.

## Current Repro

- `pnpm run build:cli:dev`: passes
- `cd tests/rspack-test && pnpm run test -t "configCases/sharing"`: passes
- `cd tests/rspack-test && pnpm run test -t "serialCases/container-1-5"`: fails in `2-layers-full`, `4-layers-full`, and `6-layers-full`

## Explorer Lane A: Bundler Runtime Ownership

- Bundler-runtime already owns the shape-sensitive `shareScopeKey` contract for container init. In core, `initContainerEntry` branches on scalar vs array at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L36`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L36) and keeps different follow-up behavior at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L87`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L87). This means the Rspack wrapper should not normalize `'default'`, `['default']`, or `[]` into one shape before delegation.
- Runtime-core already preserves remote `shareScope` shape when building remote-entry init params. `createRemoteEntryInitOptions` turns a remote’s scope into an internal array for local preparation, but emits `shareScopeKeys` back out as either the original array or the original scalar at [`/Users/zackjackson/core/packages/runtime-core/src/module/index.ts#L32`](/Users/zackjackson/core/packages/runtime-core/src/module/index.ts#L32) and [`/Users/zackjackson/core/packages/runtime-core/src/module/index.ts#L45`](/Users/zackjackson/core/packages/runtime-core/src/module/index.ts#L45).
- Bundler-runtime already owns consume/installInitialConsumes setup, including layer propagation on the resolved shared export object. `updateConsumeOptions` hydrates `moduleToHandlerMapping`, carries `shareInfo.shareConfig.layer`, and normalizes consume metadata scope arrays at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L13`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L13) and [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L41`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L41). `consumes()` and `installInitialConsumes()` then call `federationInstance.loadShare` / `loadShareSync` directly and copy the layer onto the factory result at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/consumes.ts#L24`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/consumes.ts#L24) and [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/installInitialConsumes.ts#L71`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/installInitialConsumes.ts#L71).
- Bundler-runtime also already owns sharing initialization across multiple scopes. `initializeSharing` iterates array-valued scope input itself at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L12`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L12), passes `shareScopeKeys` through to remote `init()` at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L42`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L42), and initializes external remotes from `idToRemoteMap` / `idToExternalAndNameMapping` at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L63`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initializeSharing.ts#L63).
- The one upstream limitation still visible is in `runtime-core` version-first remote matching: it filters remotes with scalar equality `remote.shareScope === shareScopeName` at [`/Users/zackjackson/core/packages/runtime-core/src/shared/index.ts#L355`](/Users/zackjackson/core/packages/runtime-core/src/shared/index.ts#L355). That is a runtime-core selection gap, not a license for the Rspack wrapper to normalize or reinterpret the bundler-runtime contract broadly.
- Conclusion: the Rspack default runtime should remain thin. It should pass through `containerShareScope`, `idToExternalAndNameMapping[*][0]`, `remoteInfos`, and consume/share metadata without rewriting array-vs-scalar shape. The current regression after removing the custom `f.consumes` wrapper points to mismatched emitted data or remote metadata, not to missing native support in bundler-runtime.

## Explorer Lane B: Core Layer Fixture and Test Parity

- The layer fixture configs are effectively in parity with core. A direct diff across `1-8-layers-full` shows only two systematic differences: Rspack imports `ModuleFederationPlugin` from `@rspack/core` instead of enhanced’s local dist build, and Rspack adds `output.module = true` in the module-output half while core relies on `experiments.outputModule = true`. There are no branch-only share-scope or layer config differences in the fixture logic itself. Evidence: [`/Users/zackjackson/core/packages/enhanced/test/configCases/layers/5-layers-full/webpack.config.js:3-18`]( /Users/zackjackson/core/packages/enhanced/test/configCases/layers/5-layers-full/webpack.config.js#L3 ), [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/5-layers-full/rspack.config.js:3-18`]( /Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/5-layers-full/rspack.config.js#L3 ), [`/Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js:3-24`]( /Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js#L3 ), [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js:3-24`]( /Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js#L3 ).
- Core’s intended layered topology is narrow and matches the current Rspack fixtures exactly in the two critical cases. Provider `5-layers-full` advertises container-level `shareScope: ['react-layer', 'default']` while the actual shared `react` registration remains only in `react-layer` at [`/Users/zackjackson/core/packages/enhanced/test/configCases/layers/5-layers-full/webpack.config.js:9-17`]( /Users/zackjackson/core/packages/enhanced/test/configCases/layers/5-layers-full/webpack.config.js#L9 ) and [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/5-layers-full/rspack.config.js:9-17`]( /Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/5-layers-full/rspack.config.js#L9 ).
- Consumer `6-layers-full` also matches core: the top-level remote `containerA` uses `shareScope: ['react-layer', 'default']`, but the consumer’s own shared `react` is constrained to `react-layer` via `layer`, `issuerLayer`, and `shareScope: 'react-layer'`. Evidence: [`/Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js:6-22`]( /Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js#L6 ), [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js:6-22`]( /Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js#L6 ).
- The only deliberate looseness in core is also preserved in Rspack: the module-output override for `6-layers-full` drops the remote `shareScope` array and only changes the external path to `../../5-layers-full/module/container.mjs`. Core does this at [`/Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js:67-77`]( /Users/zackjackson/core/packages/enhanced/test/configCases/layers/6-layers-full/webpack.config.js#L67 ), and Rspack mirrors it at [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js:68-78`]( /Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js#L68 ).
- Conclusion for the parity lane: the current failing layered serial cases are not explained by config drift between core and Rspack fixtures. The evidence points away from test-case mismatch and toward a data-emission or runtime-data-shape mismatch inside Rspack’s MF pipeline.

## Explorer Lane C: Rspack Emitted Runtime Data

- The emitted Rspack bundles show two parallel sources of shared metadata: `initializeSharingData` and the wrapper-built `initOptions.shared`. In `2-layers-full`, the generated bundle emits layer-aware sharing data at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5177`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5177), including a `"layered-components"` shared registration with `layer: "layered-components"`. The wrapper also seeds `federation.initOptions.shared` from that same data and preserves `layer` at [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L124`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L124) through [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L170`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L170).
- The consume-side emitted data is also correct and layer-aware before delegation. In `2-layers-full`, `consumesLoadingData` contains the layered React consumer with `shareScope: "layered-components"` and `layer: "layered-components"` at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5184`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5184). In `6-layers-full`, the consumer data is likewise correct with `shareScope: "react-layer"` and `layer: "react-layer"` at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L5146`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L5146).
- The strongest concrete mismatch is not in `consumesLoadingData` or remote arrays. It is the later rebake path inside bundler-runtime’s generated `updateOptions()` code, which re-reads `initializeSharingData` and calls `registerShared(shared)`, but drops `layer` from the stage object. The generated bundle for `2-layers-full` shows this directly at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L4997`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L4997) through [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5020`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/2-layers-full/main.js#L5020): the destructure only pulls `{ name, version, factory, eager, singleton, requiredVersion, strictVersion }`, then rebuilds `shareConfig` without `layer`.
- That rebake path can overwrite the correct layer-aware shared registrations that were already seeded through `initOptions.shared`. This explains why the thin runtime state regresses only on layered serial cases: generic sharing still passes, but layered registrations lose their distinguishing metadata before `loadShare()` selects the shared factory.
- Remote array share-scope data itself looks preserved where expected. In `6-layers-full`, the generated federation runtime still carries `remote_infos.containerA.shareScope = ["react-layer","default"]` at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L25`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L25), and `initializeSharingData` includes both `"default"` and `"react-layer"` remote reference stages at [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L5139`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/6-layers-full/main.js#L5139). The emitted remote array handling does not look like the first broken boundary.

## Explorer Lane D: Enhanced vs Rspack Data Pipeline

- Enhanced and Rspack diverge at the exact point that matters for this bug: enhanced does not emit `webpackRequire.initializeSharingData` in its share runtime path, while Rspack does. Enhanced’s `ShareRuntimeModule` only builds `initOptions.shared` from `share-init-option` metadata, preserving `shareConfig.layer` at [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L72`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L72) through [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L126`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L126). It then delegates `__webpack_require__.I` directly to bundler-runtime at [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L129`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L129).
- Rspack’s `ShareRuntimeModule` emits a legacy-style `initializeSharingData.scopeToSharingDataMapping` object even in enhanced mode, with layer metadata embedded in each `ProvideSharedInfo` stage at [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L96`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L96) through [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L121`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L121), and writes that object to runtime at [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L139`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L139) through [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L147`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L147).
- Bundler-runtime’s `updateOptions()` consumes that Rspack-only `initializeSharingData` object and re-registers shared modules, but it currently ignores `layer` in the rebuild path at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L103`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L103) through [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L147`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L147). Enhanced avoids this exact loss because it never feeds `initializeSharingData` into bundler-runtime in the first place.
- This explains the observed behavior cleanly:
  - enhanced passes with thin delegation because it relies on `initOptions.shared`, which already carries `layer`
  - Rspack regresses after removing the custom consumes wrapper because the thin path now allows bundler-runtime’s `updateConsumeOptions()` to consume and downgrade `initializeSharingData`
  - the issue is therefore a data-pipeline mismatch between Rspack’s emitted runtime contract and enhanced’s contract, not missing native layer support in bundler-runtime’s normal consume/remotes algorithms

## Main Synthesis

- The thin runtime direction is correct. Core already owns array share-scope handling, consume delegation, remote delegation, and container init. The layered regression appears only after Rspack takes the same thin path because Rspack still emits a legacy `initializeSharingData` contract that enhanced does not emit.
- Rspack currently seeds layer-aware `initOptions.shared` correctly in [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L124`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L124), but it also emits `initializeSharingData` from Rust and leaves it available for bundler-runtime `updateOptions()` to re-register. That rebake path strips `layer`, collapsing layered and non-layered shared registrations together.
- The most likely reason the older custom `f.consumes` wrapper made tests pass is that it changed when and how share scopes were initialized, masking this re-registration problem. It was compensating for the data-contract mismatch rather than representing the correct architecture boundary.
- The branch should keep the thin `defaultFederationRuntime` and fix the emitted data contract instead of rebuilding more wrapper logic.

## Hypotheses

- `CONFIRMED`: Generic array `shareScope` support is not the bug. Core already supports it, the Rspack wrapper now preserves it, and the failing cases are specifically layered serial cases rather than generic sharing cases. Evidence: [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L36`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/initContainerEntry.ts#L36), [`/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js#L6`](/Users/zackjackson/rspack/tests/rspack-test/serialCases/container-1-5/6-layers-full/rspack.config.js#L6), and the passing `configCases/sharing` repro.
- `CONFIRMED`: The first strong mismatch is `initializeSharingData` re-registration, not fixture drift. Enhanced fixtures and Rspack fixtures match, but only Rspack emits `initializeSharingData` in enhanced mode and thereby triggers the layer-dropping `updateOptions()` path.
- `UNRESOLVED`: The cleanest fix boundary is not yet chosen. Two plausible fixes remain:
  - stop emitting or stop reusing `initializeSharingData` for enhanced mode in Rspack, matching enhanced’s contract more closely
  - or coordinate an upstream bundler-runtime fix so `updateOptions()` preserves `layer` when rebuilding shared registrations

## Next Checks

- Verify whether marking `initializeSharingData._updated = 1` immediately after seeding `initOptions.shared` in the thin wrapper is enough to restore layered serial cases without reintroducing wrapper scope creep.
- Alternatively, test a Rust-side change that suppresses `initializeSharingData` emission for enhanced mode while preserving non-enhanced behavior.
- If either approach works locally, compare it back to enhanced’s contract and prefer the one that moves Rspack closest to enhanced rather than adding more wrapper behavior.


## Latest Delta: Module Output Remote Data

- The remaining failures after the `_updated = 1` fix are isolated to the module-output layer cases: `2-layers-full`, `4-layers-full`, and `6-layers-full`. CommonJS and non-layer sharing are green again, so the remaining bug is in the ESM remote-data handoff rather than generic shared registration.
- Core module-output bundles seed `idToRemoteMap` directly with per-module records like `{ externalType: "module", name: "" }` from `RemoteRuntimeModule`; see [`/Users/zackjackson/core/packages/enhanced/test/js/ConfigTestCases/layers/4-layers-full/module/main.mjs:5349`](/Users/zackjackson/core/packages/enhanced/test/js/ConfigTestCases/layers/4-layers-full/module/main.mjs#L5349). Rspack’s thin wrapper was reconstructing `idToRemoteMap` only from `remoteInfos[remoteName]`, which is enough for script remotes but not for module externals.
- In Rspack’s module-output bundles, `remotesLoadingData.moduleIdToRemoteDataMapping` currently carries `remoteName: "containerA"`, and the wrapper rebuilds `idToRemoteMap` from `__module_federation_remote_infos__[remoteName]`; see [`/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/4-layers-full/module/main.mjs:5209`](/Users/zackjackson/rspack/tests/rspack-test/js/serial/container-1-5/4-layers-full/module/main.mjs#L5209). That loses the direct per-module remote info core already feeds into bundler-runtime for module externals.
- Recommended fix: emit `remoteInfo` alongside each `moduleIdToRemoteDataMapping` entry in Rspack’s Rust `RemoteRuntimeModule`, using the external module’s resolved type and extracted remote name when available, and have the thin default runtime prefer that `remoteInfo` when seeding `bundlerRuntimeOptions.remotes.idToRemoteMap`. This keeps the wrapper declarative and aligns the emitted data closer to core’s `RemoteRuntimeModule` contract without reintroducing custom remote logic.

## Shared Registration Parity Note

- Scope boundary reminder:
  - Rspack still contains both legacy/V1 federation mechanisms and the current V2/enhanced federation path.
  - This branch is only supposed to change the V2/enhanced path.
  - V2 should use webpack-bundler-runtime to do the runtime work and only feed it the right data structures.
  - Any fix for the remaining layer regressions should therefore land in the enhanced plugin/runtime-module data emitters, not by thickening or compensating in `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js`.
  - Rspack V2 already has an existing `initOptions` seeding path on `main`. We should keep using that path instead of duplicating direct `federation.initOptions.shared` emission in `share_runtime_module.rs`.
  - The right enhanced-side fix is therefore to preserve `layer` in the existing emitted sharing data and mark enhanced `initializeSharingData` as already consumed (`_updated = 1`), so bundler-runtime does not rebuild downgraded registrations from the legacy structure.

- Core enhanced feeds shared registrations to bundler-runtime through `initOptions.shared`, not through `initializeSharingData`. The canonical path is [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L95`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L95) through [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L125`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L125), where `share-init-option` entries are converted into `initOptions.shared` records that preserve `shareConfig.layer`, followed by thin delegation to bundler-runtime `I(...)` at [`/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L129`](/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts#L129).
- Bundler-runtime already knows how to consume enhanced shared data once it is in the right shape. `updateConsumeOptions()` hydrates `moduleToHandlerMapping` from `consumesLoadingData`, preserves `shareInfo.shareConfig.layer`, and normalizes consume scopes without changing scalar-vs-array semantics at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L13`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L60). The extra rebuild path from `initializeSharingData` starts at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L90`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L90), and that path currently reconstructs `shareConfig` without `layer` at [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L103`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L103) through [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L148`](/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts#L148).
- Current Rspack enhanced flow diverges in two places. First, Rust still emits enhanced-mode `initializeSharingData` from [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L130`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L130) through [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L149`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L149), even though the emitted stage objects already contain `layer`, `singleton`, `requiredVersion`, and `strictVersion` at [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L96`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L96) through [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L121`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs#L121). Second, the JS wrapper rebuilds `federation.initOptions.shared` from that same legacy structure at [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L123`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L123) through [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L180`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L180), so enhanced mode currently has two shared-registration sources instead of one.
- The rest of the enhanced data plumbing is already on the right side of the boundary. `SharePlugin` only forwards `layer`, `issuerLayer`, and `request` on the enhanced path at [`/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L82`](/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L82) through [`/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L129`](/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L129), and `ConsumeSharedRuntimeModule` emits enhanced consume data with `shareScope`, `layer`, and fallback getter source already shaped for bundler-runtime at [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs#L121`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs#L121) through [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs#L173`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs#L173).
- Recommendation: for enhanced mode, Rspack should feed bundler-runtime one authoritative shared-registration source, matching core. The cleanest alignment is to stop emitting enhanced-mode `initializeSharingData` from [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs) and let bundler-runtime consume only `federation.initOptions.shared` plus `consumesLoadingData`/`remotesLoadingData`. If a smaller interim guard is needed, mark `initializeSharingData` as already consumed before bundler-runtime `updateOptions()` sees it, but that should be treated as a compatibility shim, not the target contract. This keeps `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js` thin and moves Rspack closer to the native enhanced data model bundler-runtime already expects.

## Main vs core enhanced

### Shared contract

- Core enhanced expects `federation.initOptions.shared` to be the authoritative enhanced shared-registration payload. `ShareRuntimeModule` writes entries shaped like `{ version, get, scope: string[], shareConfig, treeShaking? }` directly into `initOptions.shared` at `/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts:95-125`.
- `webpack-bundler-runtime` already consumes that shape natively in its enhanced update path. `updateConsumeOptions()` preserves `shareInfo.shareConfig.layer` and expects `scope` to already be an array-shaped list at `/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts:27-60`.
- `origin/main` Rspack matches core on the outer `shared` option shape that the thin JS wrapper reconstructs in `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js:121-174` on `origin/main`: it builds `{ version, scope: [scope], shareConfig, get, treeShaking? }`.
- `origin/main` differs from core enhanced because it still emits `__webpack_require__.initializeSharingData` from `crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs:139-147` on `origin/main` and relies on the JS wrapper to derive `initOptions.shared` from that legacy structure. Core enhanced does not carry this second shared-registration source.
- `origin/main` also differs because its Rust `ProvideSharedInfo` has no `layer` field. The struct ends at `tree_shaking_mode` in `crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs:178-186` on `origin/main`, while core enhanced distinguishes same-version shared registrations by `shareConfig.layer` at `/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ShareRuntimeModule.ts:81-89`.

### Provide contract

- Core enhanced provide normalization outputs:
  - `shareScope`
  - `shareKey`
  - `version`
  - `eager`
  - `requiredVersion`
  - `strictVersion`
  - `singleton`
  - `layer`
  - `request`
  - `treeShakingMode`
  at `/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ProvideSharedPlugin.ts:105-121`.
- `origin/main` Rspack `ProvideSharedPlugin.ts` already matches the non-layer subset for enhanced mode in `packages/rspack/src/sharing/ProvideSharedPlugin.ts:63-79` on `origin/main`:
  - `shareScope`
  - `shareKey`
  - `version`
  - `eager`
  - `singleton`
  - `requiredVersion`
  - `strictVersion`
  - `treeShakingMode`
- The exact main-vs-core gaps in provide normalization are `layer` and `request`. Those are the real enhanced-only additions this file needs; the rest of the contract is already aligned enough with core.

### Consume contract

- Core enhanced emits `consumesLoadingData.moduleIdToConsumeDataMapping[moduleId]` entries with:
  - `fallback`
  - `shareScope: string[]`
  - `singleton`
  - `requiredVersion`
  - `strictVersion`
  - `eager`
  - `layer`
  - `shareKey`
  - `treeShakingMode?`
  at `/Users/zackjackson/core/packages/enhanced/src/lib/sharing/ConsumeSharedRuntimeModule.ts:79-94`.
- `webpack-bundler-runtime` expects that shape and normalizes `scope` with `Array.isArray(data.shareScope) ? data.shareScope : [data.shareScope || 'default']` at `/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts:49-52`.
- `origin/main` Rspack already matches most of that consume contract in `crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs:116-127` on `origin/main`:
  - `shareScope`
  - `shareKey`
  - `import`
  - `requiredVersion`
  - `strictVersion`
  - `singleton`
  - `eager`
  - `fallback`
  - `treeShakingMode`
- The exact main-vs-core gaps are:
  - no emitted `layer` field in `CodeGenerationDataConsumeShared` or the generated consume mapping; the struct ends at `tree_shaking_mode` in `crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs:239-250` on `origin/main`
  - the thin JS wrapper rebuilds `shareInfo.scope` as `[data.shareScope]` in `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js:87-106` on `origin/main`, which does not faithfully preserve enhanced array consume scopes

### Remotes contract

- Core enhanced directly seeds the bundler-runtime remotes contract with:
  - `chunkMapping`
  - `idToExternalAndNameMapping`
  - `idToRemoteMap`
  at `/Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts:151-175`.
- Core’s `idToRemoteMap[moduleId]` entries are per-module remote descriptors like `{ externalType, name }` at `/Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts:129-142`.
- `webpack-bundler-runtime` can only backfill `idToRemoteMap` from `remoteInfos[data.remoteName]` if no direct per-module map exists, at `/Users/zackjackson/core/packages/webpack-bundler-runtime/src/updateOptions.ts:178-185`.
- `origin/main` Rspack already matches core on the lower remotes payload in `crates/rspack_plugin_mf/src/container/remote_runtime_module.rs:82-95` and `:123-129` on `origin/main`:
  - `remotesLoadingData.chunkMapping`
  - `remotesLoadingData.moduleIdToRemoteDataMapping` entries with `shareScope`, `name`, `externalModuleId`, and `remoteName`
- `origin/main` differs from core enhanced because it does not emit per-module `remoteInfo`/`idToRemoteMap` data from Rust. The JS wrapper rebuilds `idToRemoteMap` only from `__module_federation_remote_infos__[remoteName]` in `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js:225-238` on `origin/main`, which is weaker than core’s direct per-module remote descriptors.

### Thin JS runtime wrapper

- Core enhanced keeps the runtime wrapper thin:
  - `ShareRuntimeModule` writes `initOptions.shared` and delegates `I(...)`
  - `ConsumeSharedRuntimeModule` writes `consumesLoadingData` and delegates `consumes(...)`
  - `RemoteRuntimeModule` writes remote data and delegates `remotes(...)`
- `origin/main` Rspack already matches that architecture at a high level in `packages/rspack/src/runtime/moduleFederationDefaultRuntime.js:175-320` on `origin/main`:
  - delegates `remotes`
  - delegates `consumes`
  - delegates `I`
  - delegates `initContainer`
  - initializes via `bundlerRuntime.init`
- The enhanced-only mismatches in the thin wrapper are data-shape mismatches, not missing runtime hooks:
  - wrapper-built `shared` cannot preserve `layer` because `initializeSharingData` on main has no place for it
  - wrapper-built consume handler data wraps `shareScope` as `[data.shareScope]` instead of preserving arrays
  - wrapper-built `idToRemoteMap` has no per-module remote descriptor data

### Bottom line

- For enhanced mode, core expects bundler-runtime to receive:
  - shared registrations through `initOptions.shared`
  - consume metadata through `consumesLoadingData.moduleIdToConsumeDataMapping`
  - remote metadata through `chunkMapping`, `idToExternalAndNameMapping`, and `idToRemoteMap`
- `origin/main` Rspack already has the right high-level architecture and most of the non-layer enhanced fields.
- The exact enhanced-only gaps relative to core are:
  - add `layer` to provide shared data
  - add `layer` to consume shared data
  - preserve enhanced consume `shareScope` as the array shape bundler-runtime expects
  - add per-module remote info equivalent to core’s `idToRemoteMap` entries
  - stop treating `initializeSharingData` as an enhanced shared-registration source once `initOptions.shared` is available

## Worktree vs main

Current dirty MF files outside `tests/`:

- [`/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs)
- [`/Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts)
- [`/Users/zackjackson/rspack/packages/rspack/src/container/ContainerReferencePlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/container/ContainerReferencePlugin.ts)
- [`/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js)
- [`/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts)

### Keep

- Keep the `remote_info` emission in [`remote_runtime_module.rs:82`]( /Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs#L82 ) through [`remote_runtime_module.rs:112`]( /Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs#L112 ) and [`remote_runtime_module.rs:158`]( /Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs#L158 ) through [`remote_runtime_module.rs:170`]( /Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs#L170 ). This is the right emitter-side parity fix with core enhanced `RemoteRuntimeModule`, which computes per-module remote records from the resolved `ExternalModule` at [`/Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts:98`]( /Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts#L98 ) through [`/Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts:142`]( /Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts#L142 ).
- Keep layer pass-through in the JS runtime wrapper where it is only translating already-emitted data into bundler-runtime input:
  - consume handler `shareConfig.layer` in [`moduleFederationDefaultRuntime.js:87`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L87 ) through [`moduleFederationDefaultRuntime.js:97`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L97 )
  - `initOptions.shared[*].shareConfig.layer` in [`moduleFederationDefaultRuntime.js:127`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L127 ) through [`moduleFederationDefaultRuntime.js:168`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L168 )
  This is not new policy; `moduleFederationDefaultRuntime.js` already owns the translation from emitted `initializeSharingData` / `consumesLoadingData` into bundler-runtime options on `main`.
- Keep the already-established enhanced-only plugin/config plumbing that is net-different from `main` but not just local dirt:
  - expose `layer` pass-through in [`ContainerPlugin.ts:31`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts#L31 ) through [`ContainerPlugin.ts:66`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts#L66 )
  - enhanced-only `layer`, `issuerLayer`, and `request` forwarding in [`SharePlugin.ts:41`]( /Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L41 ) through [`SharePlugin.ts:147`]( /Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L147 )
  Those changes are part of the plugin-side data pipeline for enhanced layer support, not wrapper logic.

### Revert to main

- Revert the local `validateShareScope` churn:
  - helper introduction in [`SharePlugin.ts:13`]( /Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L13 ) through [`SharePlugin.ts:23`]( /Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts#L23 )
  - constructor-time validation in [`ContainerPlugin.ts:43`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts#L43 ) through [`ContainerPlugin.ts:45`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts#L45 )
  - constructor/remotes validation in [`ContainerReferencePlugin.ts:40`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerReferencePlugin.ts#L40 ) through [`ContainerReferencePlugin.ts:67`]( /Users/zackjackson/rspack/packages/rspack/src/container/ContainerReferencePlugin.ts#L67 )
  This is local review-churn, not required for enhanced layer support. It broadens JS plugin behavior beyond `main` without affecting the actual emitter/runtime data contract.
- `ContainerReferencePlugin.ts` is currently dirty only relative to `HEAD`; it has no net diff vs `origin/main`. That means it should just be reverted/cleaned unless there is a deliberate reason to keep it dirty.

### Move/keep in emitters, not wrapper logic

- Remote metadata belongs in emitters/runtime modules, not in JS wrapper inference. The Rust `remote_info` addition is the right place for this data. The wrapper should only consume it when present. Right now the dirty wrapper regressed to `main`-style lookup at [`moduleFederationDefaultRuntime.js:233`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L233 ) through [`moduleFederationDefaultRuntime.js:240`]( /Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js#L240 ), which ignores emitted `remote_info`. That is inconsistent with the emitter change and should be corrected by preferring emitted remote data, not by adding more wrapper-side reconstruction logic.
- The exact mismatch is with bundler-runtime’s `idToRemoteMap` expectations in [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/remotes.ts:27`]( /Users/zackjackson/core/packages/webpack-bundler-runtime/src/remotes.ts#L27 ) through [`/Users/zackjackson/core/packages/webpack-bundler-runtime/src/remotes.ts:145`]( /Users/zackjackson/core/packages/webpack-bundler-runtime/src/remotes.ts#L145 ): it reads per-module `remoteInfos` from `idToRemoteMap[id]`. Core enhanced seeds those per-module records directly in [`/Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts:112`]( /Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts#L112 ) through [`RemoteRuntimeModule.ts:142`]( /Users/zackjackson/core/packages/enhanced/src/lib/container/RemoteRuntimeModule.ts#L142 ). Rspack should match that by emitting the data in Rust and letting the wrapper pass it through.
- Layer metadata belongs primarily in plugin/runtime-module emitted data, with the JS wrapper only forwarding it into bundler-runtime-owned structures. The authoritative source should stay in Rust sharing emitters and JS plugins, not accumulate new runtime policy in `moduleFederationDefaultRuntime.js`.

### Net call

- Necessary current local change: `remote_runtime_module.rs` `remote_info` emission.
- Necessary but thin wrapper pass-through: `layer` forwarding in `moduleFederationDefaultRuntime.js`.
- Revert current local churn: `validateShareScope` helper and its `ContainerPlugin` / `ContainerReferencePlugin` call sites.
- Fix the current emitter/wrapper mismatch by restoring wrapper consumption of emitted `remote_info`; do not replace it with new wrapper-generated remote metadata.

## Current non-test diff audit

Scope of this audit:

- Compare `origin/main...HEAD`
- Limit to `crates/` and `packages/`
- Ignore all `tests/` changes
- Treat `moduleFederationDefaultRuntime.js` as correct only when it stays at `main` behavior plus `layer` pass-through

### JS and wrapper changes that are still broader than the minimum

- [`packages/rspack/src/runtime/moduleFederationDefaultRuntime.js`](/Users/zackjackson/rspack/packages/rspack/src/runtime/moduleFederationDefaultRuntime.js) is now close to the right boundary. Its net diff is only `layer` pass-through on the existing main-branch translation paths:
  - consume handler `shareConfig.layer`
  - `initOptions.shared[*].shareConfig.layer`
  This is in scope because `main` already uses this wrapper to translate emitted runtime data into bundler-runtime inputs. No extra array-share-scope normalization or custom consume/remotes policy should be added here.
- The broader JS sharing surface is still wider than the minimum needed for the layer fix:
  - [`packages/rspack/src/sharing/SharePlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/SharePlugin.ts)
  - [`packages/rspack/src/sharing/ConsumeSharedPlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/ConsumeSharedPlugin.ts)
  - [`packages/rspack/src/sharing/ProvideSharedPlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/ProvideSharedPlugin.ts)
  - [`packages/rspack/src/sharing/utils.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/utils.ts)
  - [`packages/rspack/src/sharing/CollectSharedEntryPlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/sharing/CollectSharedEntryPlugin.ts)
- What is broader than necessary in those files:
  - new helper normalization like `resolveShareRequest`, `resolveShareKey`, and `resolveShareScope`
  - extra enhanced-mode branching around defaults that `main` already knows how to apply
  - enhanced-only normalization in `CollectSharedEntryPlugin`, which is manifest/optimizer support, not the minimum needed to make the layer serial cases pass
- Minimum JS/plugin change should be narrower:
  - keep `main` defaulting behavior
  - only thread enhanced-only fields that do not exist on `main`: `layer`, `issuerLayer`, and explicit `request`

### Non-test changes that do not look necessary for the layer serial runtime fix

- [`crates/rspack_plugin_javascript/Cargo.toml`](/Users/zackjackson/rspack/crates/rspack_plugin_javascript/Cargo.toml) is pure churn.
- Manifest layer metadata is useful branch work, but it is not part of the minimum fix for the thin-wrapper layer serial failures:
  - [`crates/rspack_plugin_mf/src/manifest/data.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/manifest/data.rs)
  - [`crates/rspack_plugin_mf/src/manifest/mod.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/manifest/mod.rs)
- Shared-entry / optimizer identifier parsing is also not part of the minimum runtime parity fix:
  - [`crates/rspack_plugin_mf/src/sharing/collect_shared_entry_plugin.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/collect_shared_entry_plugin.rs)
  - [`crates/rspack_plugin_mf/src/sharing/shared_used_exports_optimizer_plugin.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/shared_used_exports_optimizer_plugin.rs)

### Minimum emitter and data changes that still look justified

- Raw/binding plumbing for enhanced-only data is in scope:
  - [`crates/rspack_binding_api/src/raw_options/raw_builtins/raw_mf.rs`](/Users/zackjackson/rspack/crates/rspack_binding_api/src/raw_options/raw_builtins/raw_mf.rs)
  - [`crates/node_binding/napi-binding.d.ts`](/Users/zackjackson/rspack/crates/node_binding/napi-binding.d.ts)
  These fields are the minimum bridge needed so enhanced options can carry `layer`, `issuerLayer`, and `request` into the Rust MF plugins.
- Expose-layer plumbing is in scope:
  - [`crates/rspack_plugin_mf/src/container/container_plugin.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_plugin.rs)
  - [`crates/rspack_plugin_mf/src/container/container_entry_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_entry_module.rs)
  - [`crates/rspack_plugin_mf/src/container/container_exposed_dependency.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_exposed_dependency.rs)
  - [`packages/rspack/src/container/ContainerPlugin.ts`](/Users/zackjackson/rspack/packages/rspack/src/container/ContainerPlugin.ts)
- Shared consume/provide layer data in Rust is in scope because this is the authoritative data path bundler-runtime actually consumes:
  - [`crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs)
  - [`crates/rspack_plugin_mf/src/sharing/provide_shared_plugin.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/provide_shared_plugin.rs)
  - [`crates/rspack_plugin_mf/src/sharing/consume_shared_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_module.rs)
  - [`crates/rspack_plugin_mf/src/sharing/provide_shared_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/provide_shared_module.rs)
  - [`crates/rspack_plugin_mf/src/sharing/consume_shared_fallback_dependency.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_fallback_dependency.rs)
  - [`crates/rspack_plugin_mf/src/sharing/provide_shared_dependency.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/provide_shared_dependency.rs)
  - [`crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs)
- [`crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/share_runtime_module.rs) should stay very close to `main`. The justified delta is only adding `ProvideSharedInfo.layer` into the existing emitted structure, not inventing a new shared-registration contract in the wrapper.

### Most likely remaining minimum fix point

- The current thin-wrapper failures are concentrated in `2-layers-full`, `4-layers-full`, and `6-layers-full`.
- Those are the module-output remote cases, which points more strongly at remote metadata than at generic shared registration.
- Core enhanced seeds richer per-module remote runtime data directly from `RemoteRuntimeModule.ts`, including explicit per-module `idToRemoteMap` entries for each remote external.
- Rspack’s minimum remaining parity fix therefore looks like:
  - keep the wrapper at `main` plus `layer` pass-through
  - keep wrapper consumption of emitted per-module `remoteInfo` when present
  - finish the Rust-side remote emitter so module externals provide the per-module remote metadata bundler-runtime expects
- That points directly at [`crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`](/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs) as the strongest remaining bug site.

### Bottom line

- The remaining layer regression should be solved by emitter/data parity, not by adding more JS runtime compensation.
- The minimum branch shape is:
  - `main`-style wrapper
  - `layer` added to the existing consume/provide emitted data
  - per-module remote info emitted from Rust for module externals
- Everything beyond that in JS sharing normalization and manifest/optimizer support is separate branch scope, not the minimum needed to make the layer serial cases pass.
