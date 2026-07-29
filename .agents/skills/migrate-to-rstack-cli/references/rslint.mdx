# Rslint Migration

Read this reference when the project uses `@rslint/core`, `rslint.config.*`, `rslint` commands, or Rslint config imports.

## Steps

1. Replace the `rslint` executable prefix with `rs lint`. For example, replace `rslint --fix` with `rs lint --fix`.
2. Move the old config into `define.lint`, replacing Rslint's `defineConfig()` wrapper and import. Dynamically import presets from `rstack/lint` inside an async config function.
3. Replace direct config/API imports from `@rslint/core` with exports from `rstack/lint` where available.
4. Replace custom `--config` paths with the migrated `rstack.config.*` path.
5. Remove `@rslint/core` only when no uncovered direct runtime API remains. Delete `rslint.config.*`.

## Config Pattern

```ts
import { define } from 'rstack';

define.lint(async () => {
  const { js, ts } = await import('rstack/lint');
  return [js.configs.recommended, ts.configs.recommended];
});
```

## Script Pattern

```json
{
  "scripts": {
    "lint": "rs lint && prettier -c .",
    "lint:write": "rs lint --fix && prettier -w ."
  }
}
```

## Validate

Run the non-writing lint script.
