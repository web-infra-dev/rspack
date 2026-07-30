# Rspress Migration

Read this reference when the project uses `@rspress/core`, `rspress.config.*`, `rspress` commands, docs config, themes, or plugins.

## Steps

1. Keep `@rspress/core` installed as a direct dependency. `rs doc` requires this optional Rstack peer dependency.
2. Replace the `rspress` executable prefix with `rs doc`: `rspress dev` to `rs doc`, `rspress build` to `rs doc build`, and `rspress preview` to `rs doc preview`. Preserve supported arguments.
3. Rename or replace the `rspress.config.*` with `rstack.config.*`, then move its export into `define.doc`, replacing Rspress's `defineConfig()` wrapper and import. Preserve async functions, themes, plugins, and all options.
4. Replace custom `--config` paths with the migrated `rstack.config.*` path.
5. Keep Rspress-specific type imports from `@rspress/core`; Rstack does not re-export them.
6. Keep themes, plugins, and docs UI packages. Delete `rspress.config.*` only after the migrated docs commands load equivalent config.

## Config Pattern

```ts
import { define } from 'rstack';

define.doc({
  root: 'docs',
  title: 'Project docs',
});
```

Use an async config and dynamic imports when plugins or themes require runtime imports.

## Validate

Run the migrated docs build script. Smoke-test dev or preview when those workflows changed.
