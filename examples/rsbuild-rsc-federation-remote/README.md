# Rsbuild RSC + Module Federation manifest example (remote variant)

This package mirrors the base `rsbuild-rsc-federation` example and runs on a separate port for dual-app testing.

## Run

From repository root:

```bash
pnpm --filter examples-rsbuild-rsc-federation-remote... install
pnpm --filter examples-rsbuild-rsc-federation-remote run build
pnpm --filter examples-rsbuild-rsc-federation-remote run verify:manifest
```

## E2E smoke test

For the combined dual-app Playwright run (both apps booted before test execution), use:

```bash
pnpm --filter examples-rsbuild-rsc-federation run test:e2e
```

This package also supports standalone e2e:

```bash
pnpm --filter examples-rsbuild-rsc-federation-remote... install
pnpm --filter examples-rsbuild-rsc-federation-remote run test:e2e
```
