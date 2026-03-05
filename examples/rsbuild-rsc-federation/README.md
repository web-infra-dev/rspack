# Rsbuild RSC + Module Federation manifest example

This example is based on the [`rsbuild-plugin-rsc` `examples/server`](https://github.com/rstackjs/rsbuild-plugin-rsc/tree/main/examples/server) project structure and extends it with Module Federation manifest output validation.

## Goals

- Build an Rsbuild app with React Server Components (RSC) and Module Federation.
- Verify emitted `mf-stats.json` and `mf-manifest.json` contain expected RSC metadata for:
  - `shared` lookup by `shareKey`
  - `exposes` lookup by `container/exposeKey`
  - `remotes` lookup by `remote/moduleName`
- Ensure key React runtime packages are configured as singletons for layered RSC/SSR usage.

## Shared singleton matrix

Configured in `rsbuild.config.ts` under MF `shared`:

- `react` (`singleton: true`, RSC layer)
- `react/jsx-runtime` (`singleton: true`, RSC layer)
- `react-dom` (`singleton: true`, SSR layer)
- `react-dom/server` (`singleton: true`, SSR layer)
- `react-server-dom-rspack/server.node` (`singleton: true`, RSC layer)
- `rsbuild-rsc-federation-shared` (workspace package, `shareKey: "rsc-shared-key"`, RSC layer)

## Run

From repository root:

```bash
pnpm --filter examples-rsbuild-rsc-federation... install
pnpm --filter examples-rsbuild-rsc-federation run build
pnpm --filter examples-rsbuild-rsc-federation run verify:manifest
```

The verification script fails fast on any mismatch and prints the resolved manifest file paths on success.
