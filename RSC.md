# RSC in Rspack: Full Repo Breakdown

RSC in this repo is implemented as a coordinated multi-compiler pipeline (server + client), with SWC directive transforms feeding metadata into a Rust plugin that injects entries/loaders and emits a runtime manifest.

## 1) Public API Surface (What Users Configure)

- `experiments.rsc` is exported at `/Users/zackjackson/rspack/packages/rspack/src/exports.ts:400` and wired from `/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/index.ts:14`.
- `createPlugins()` returns paired `ServerPlugin` + `ClientPlugin` that share one coordinator (`/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/index.ts:21`).
- Layer constants are exposed as `Layers.rsc = "react-server-components"` and `Layers.ssr = "server-side-rendering"` (`/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/index.ts:41`).
- SWC opt-in is `rspackExperiments.reactServerComponents` (`/Users/zackjackson/rspack/packages/rspack/src/builtin-loader/swc/types.ts:32`).

## 2) JS-Side Coordinator and Watch Behavior

- JS `Coordinator` bridges to Rust `JsCoordinator`, provides server compiler ID, and synchronizes watch behavior (`/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/Coordinator.ts:9`).
- Server compiler proxies client dependencies into server watched deps (`/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/Coordinator.ts:45`).
- Client compiler watch is forced to ignore all files; server invalidation drives client invalidation (`/Users/zackjackson/rspack/packages/rspack/src/builtin-plugin/rsc/Coordinator.ts:77`).

## 3) Binding and Plugin Registration

- NAPI bridge types for RSC plugins/coordinator: `/Users/zackjackson/rspack/crates/rspack_binding_api/src/plugins/rsc.rs:21`.
- Builtin plugin names include `RscServerPlugin` and `RscClientPlugin` (`/Users/zackjackson/rspack/crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs:252`).
- Loader plugins for RSC are always registered in compiler bootstrap: `ClientEntryLoaderPlugin` and `ActionEntryLoaderPlugin` (`/Users/zackjackson/rspack/crates/rspack_binding_api/src/lib.rs:204`).

## 4) Core RSC Metadata Model

- `RscModuleType` (`ServerEntry | Server | Client`) and `RscMeta` live in `/Users/zackjackson/rspack/crates/rspack_core/src/module.rs:56`.
- Metadata fields: `server_refs`, `client_refs`, `is_cjs`, `action_ids` (`/Users/zackjackson/rspack/crates/rspack_core/src/module.rs:72`).
- This metadata is stored on `BuildInfo.rsc` (`/Users/zackjackson/rspack/crates/rspack_core/src/module.rs:113`).

## 5) SWC RSC Transform Pipeline (Directive/Compiler Semantics)

- RSC pass entrypoint: `/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/mod.rs:18`.
- It detects `react-server-components` layer and skips duplicate metadata generation for `?rsc-server-entry-proxy=true` (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/mod.rs:26`).
- It runs two transforms: `react_server_components` + `server_actions` (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/mod.rs:35`).
- SWC loader enables this only when `rspackExperiments.reactServerComponents` is true (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/lib.rs:149`).

## 6) `react_server_components` Transform

- Parses directives/imports/exports and validates directive placement (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/react_server_components.rs:208`).
- Sets `RscMeta.module_type` to `ServerEntry` for `"use server-entry"` or `Client` for `"use client"` when in RSC layer (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/react_server_components.rs:121`).
- Removes top-level `"use client"` directive from AST (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/react_server_components.rs:106`).
- Validates disallowed server-side React APIs/hooks (for example `useState`, `useEffect`) for server graph (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/react_server_components.rs:396`).

## 7) `to_module_ref` Rewriting

- After metadata is attached, RSC-layer modules may be replaced with generated proxy modules (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/to_module_ref.rs:186`).
- Server-entry modules become `createServerEntry(...)` wrappers (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/to_module_ref.rs:46`).
- Client boundary modules become `registerClientReference(...)` wrappers that throw if invoked on server (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/to_module_ref.rs:86`).
- `export *` is explicitly rejected for server entry and client boundary proxies (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/to_module_ref.rs:205`).

## 8) Server Actions Transform

- Main file: `/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:100`.
- Detects file/function `"use server"` directives with strict placement and typo diagnostics (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:2311`).
- Enforces async-only server actions and multiple safety constraints (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:2539`).
- Generates deterministic action IDs (hash of salt + file + export, plus arg mask bits) (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:198`).
- Writes `action_ids` into `RscMeta` (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:763`).
- In server layer: injects `registerServerReference`, optional encrypt/decrypt bound arg helpers, and `ensureServerActions` (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:1762`).
- In non-server layer: exports are converted to `createServerReference(actionId)` stubs (`/Users/zackjackson/rspack/crates/rspack_loader_swc/src/rsc_transforms/server_actions.rs:1563`).

## 9) Rust RSC Plugin Orchestration

- Crate root: `/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/lib.rs:1`.
- Shared state per compiler (`PLUGIN_STATES`) tracks manifests, injected entries, CSS/JS per entry, action maps, and changed server components (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/plugin_state.rs:13`).
- Rust coordinator enforces state machine: `Idle -> ServerEntries -> ClientEntries -> ServerActions -> Idle` (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/coordinator.rs:20`).
- `RscServerPlugin`:
  - resets/initializes state at `this_compilation` (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/server_plugin.rs:88`),
  - tracks changed server components (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/server_plugin.rs:117`),
  - collects component info and injects client/SSR/action entries (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/server_plugin.rs:186`),
  - emits runtime module when `RSC_MANIFEST` is required (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/server_plugin.rs:131`),
  - triggers `onServerComponentChanges` when diffs exist (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/server_plugin.rs:540`).
- `RscClientPlugin`:
  - waits for server stage, injects client entries from server-collected state (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/client_plugin.rs:487`),
  - traverses chunk graph to build client manifest + entry CSS/JS lists (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/client_plugin.rs:370`),
  - collects client-side discovered actions for server follow-up injection (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/client_plugin.rs:586`).

## 10) Internal Loaders Used by Server/Client Plugins

- Client entry loader parses query `modules` + `server=true|false` and emits imports (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/loaders/client_entry_loader.rs:62`).
- For server-side injected client entries it uses eager imports and strips CSS (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/loaders/client_entry_loader.rs:114`).
- Action entry loader parses action map and emits re-exports keyed by action ID (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/loaders/action_entry_loader.rs:42`).
- Both are resolved by dedicated loader plugins in normal module factory hooks (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/loaders/client_entry_loader_plugin.rs:41`, `/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/loaders/action_entry_loader_plugin.rs:41`).

## 11) Runtime Manifest Plumbing

- Parser exposes `__rspack_rsc_manifest__` and adds runtime requirement `RuntimeGlobals::RSC_MANIFEST` (`/Users/zackjackson/rspack/crates/rspack_plugin_javascript/src/parser_plugin/api_plugin.rs:57` and `:266`).
- Runtime global `RSC_MANIFEST` maps to `__webpack_require__.rscM` (`/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs:475`).
- Runtime module builds and injects manifest object at runtime (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/manifest_runtime_module.rs:67` and `:114`).
- Manifest includes server actions map, client module map, SSR consumer map, module loading config, and entry CSS/JS files (`/Users/zackjackson/rspack/crates/rspack_plugin_rsc/src/manifest_runtime_module.rs:38`).

## 12) Existing RSC Tests in Repo

- `server-entry` case verifies `"use server-entry"` artifacts expose `entryJsFiles`/`entryCssFiles` (`/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/server-entry/src/framework/entry.rsc.js:8`).
- `server-actions-production` verifies all server actions survive production build and are loadable (`/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/server-actions-production/src/framework/entry.rsc.js:11`).
- `with-concatenated-module` verifies manifest correctness with module concatenation + action loading + consumer module map (`/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/with-concatenated-module/src/framework/entry.rsc.js:11`).
- All three fixtures use dual compiler configs, `experiments.rsc.createPlugins()`, layer assignment, and SWC `reactServerComponents` enablement (`/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/*/rspack.config.js`).

## 13) Examples and E2E Sample Locations

- In-repo runnable RSC sample fixtures are the `configCases/rsc-plugin` cases:
  - `/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/server-entry`
  - `/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/server-actions-production`
  - `/Users/zackjackson/rspack/tests/rspack-test/configCases/rsc-plugin/with-concatenated-module`
- Main docs walkthrough and configuration guide:
  - `/Users/zackjackson/rspack/website/docs/en/guide/tech/rsc.mdx`
  - `/Users/zackjackson/rspack/website/docs/zh/guide/tech/rsc.mdx`
- External full app examples referenced by docs:
  - `https://github.com/rstackjs/rspack-rsc-examples`
- Dedicated `tests/e2e` RSC cases are not present in this repo snapshot; the primary automated RSC validation here is under `tests/rspack-test/configCases/rsc-plugin`.

This is a static code-path map of how RSC is implemented and wired in the current repo snapshot.
