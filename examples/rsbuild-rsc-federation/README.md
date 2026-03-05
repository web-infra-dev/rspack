# Rsbuild RSC + Module Federation manifest example

This example is based on the [`rsbuild-plugin-rsc` `examples/server`](https://github.com/rstackjs/rsbuild-plugin-rsc/tree/main/examples/server) project structure and extends it with Module Federation manifest output validation.

## Goals

- Build an Rsbuild app with React Server Components (RSC) and Module Federation.
- Verify emitted `mf-stats.json` and `mf-manifest.json` contain expected RSC metadata for:
  - `shared` lookup by `shareKey`
  - `exposes` lookup by `container/exposeKey`
  - `remotes` lookup by `remote/moduleName`
- Ensure key React runtime packages are configured as singletons for layered RSC/SSR usage.
- Ensure Module Federation plugin is applied to **both** Rsbuild environments:
  - server (`target: node`)
  - client (`target: web`)

## Host/remote topology

This package is the **host** app. It consumes the remote app at:

- server runtime: `remote@http://localhost:3331/mf-manifest.json`
- client runtime: `remote@http://localhost:3331/mf-manifest.client.json`

`tools.rspack` injects `ModuleFederationPlugin` for both environments with environment-specific options:

- **Server build**
  - container: `rsbuild_host`
  - entry: `remoteEntry.js`
  - manifest files: `mf-stats.json` and `mf-manifest.json`
  - exposes: `./button`, `./composed`, `./consumer`, `./server-mixed`
  - runtime plugins:
    - `@module-federation/node/runtimePlugin`
    - `./src/framework/mf-rsc-registration-runtime-plugin.js`
- **Client build**
  - container: `rsbuild_host`
  - entry: `remoteEntry.client.js`
  - manifest files: `mf-manifest.client-stats.json` and `mf-manifest.client.json`
  - exposes: `./button`, `./composed`
  - runtime plugins:
    - `./src/framework/mf-rsc-registration-runtime-plugin.js`

## Dynamic RSC registration runtime plugin

The host includes a dedicated MF runtime plugin that dynamically registers
remote/shared RSC metadata into `__webpack_require__.rscM` using MF runtime
lifecycle hooks (`afterResolve`, `onLoad`) and `mf-manifest.json` data.

This implementation intentionally avoids exposing `__webpack_require__` on
global scopes.

## Shared singleton matrix

Configured in `rsbuild.config.ts` under MF `shared`:

Server:

- `react` (`singleton: true`, RSC layer)
- `react/jsx-runtime` (`singleton: true`, RSC layer)
- `react-dom` (`singleton: true`, SSR layer)
- `react-dom/server` (`singleton: true`, SSR layer)
- `react-server-dom-rspack/server.node` (`singleton: true`, RSC layer)
- `rsbuild-rsc-federation-shared` (workspace package, `shareKey: "rsc-shared-key"`, RSC layer)
- `rsbuild-rsc-federation-shared/server-actions` (workspace package subpath, `shareKey: "rsc-shared-actions-key"`, RSC layer)

Client:

- `react` (`singleton: true`, client scope)
- `react/jsx-runtime` (`singleton: true`, client scope)
- `react-dom` (`singleton: true`, client scope)

## Run

From repository root:

```bash
pnpm --filter examples-rsbuild-rsc-federation... install
pnpm --filter examples-rsbuild-rsc-federation run build
pnpm --filter examples-rsbuild-rsc-federation run verify:manifest
```

The verification script fails fast on any mismatch and prints the resolved manifest file paths on success.

## E2E smoke test

This example includes a Playwright e2e that starts **both** app examples first:

- `http://localhost:3330` (`examples/rsbuild-rsc-federation`)
- `http://localhost:3331` (`examples/rsbuild-rsc-federation-remote`)

Then it validates for each app:

- app renders (`client entry ready`)
- client component text renders (`InteractiveClientDemo`)
- client interactivity works (counter increments on button clicks)

Run:

```bash
pnpm --filter examples-rsbuild-rsc-federation... install
pnpm --filter examples-rsbuild-rsc-federation run test:e2e
```
