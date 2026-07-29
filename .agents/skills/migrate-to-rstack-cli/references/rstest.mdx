# Rstest Migration

Read this reference when the project uses `@rstest/core`, `@rstest/adapter-rsbuild`, `@rstest/adapter-rslib`, `rstest.config.*`, `rstest` commands, or Rstest imports and types.

## Steps

1. Replace the `rstest` executable prefix with `rs test`. Preserve supported arguments, such as `-w`, after `rs test`.
2. Move test options into `define.test`, replacing Rstest's `defineConfig()` wrapper and import.
3. Remove standard `withRsbuildConfig()` or `withRslibConfig()` wiring only when `define.app` or `define.lib` is registered by the same `rstack.config.*` loaded by `rs test`. Rstack derives that extension unless `define.test` sets `extends`.
4. Preserve explicit custom `extends` values. Keep any adapter or package still imported by that custom extension.
5. Replace imports and TypeScript `types` entries:
   - `@rstest/core` to `rstack/test`
   - `@rstest/core/globals` to `rstack/test/globals`
   - `@rstest/core/importMeta` to `rstack/test/importMeta`
6. Search for remaining direct core or adapter imports. Remove `@rstest/core` and adapter dependencies only when no direct use remains.
7. Delete `rstest.config.*` after all behavior is represented or intentionally supplied by automatic app/library extension.

## Config Pattern

```ts
import { define } from 'rstack';

define.test({
  setupFiles: ['./tests/setup.ts'],
  testEnvironment: 'happy-dom',
});
```

## Validate

Run the migrated test script. Follow repository requirements such as building before tests.
