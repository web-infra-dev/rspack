---
name: rstack-cli-best-practices
description: Guidance on using Rstack CLI, including `rs` commands, the `rstack.config.ts` file, and import paths from the `rstack` package. Use for Rstack CLI-related tasks.
---

# Rstack CLI Best Practices

Rstack CLI is the `rstack` package, exposed through the `rs` binaries. It provides one CLI, one config file, and a consistent workflow for the Rstack JavaScript toolchain.

It covers web app, library, docs, test, lint, Git hook, and staged-file workflows.

## Commands

Use `rs -h` for top-level help, and `rs <command> -h` for command help where supported.

| Command      | Purpose                          | Underlying tool | Config          |
| ------------ | -------------------------------- | --------------- | --------------- |
| `rs dev`     | Run the app dev server           | Rsbuild         | `define.app`    |
| `rs build`   | Build the app for production     | Rsbuild         | `define.app`    |
| `rs preview` | Preview the app production build | Rsbuild         | `define.app`    |
| `rs lib`     | Build a library                  | Rslib           | `define.lib`    |
| `rs doc`     | Serve or build docs              | Rspress         | `define.doc`    |
| `rs test`    | Run tests                        | Rstest          | `define.test`   |
| `rs lint`    | Lint code                        | Rslint          | `define.lint`   |
| `rs setup`   | Install project-local Git hooks  | None            | None            |
| `rs staged`  | Run tasks on staged Git files    | lint-staged     | `define.staged` |

Key behavior:

- Unless `define.test` already sets `extends`, `rs test` extends `define.app` through `@rstest/adapter-rsbuild` or falls back to `define.lib` through `@rstest/adapter-rslib`. The app config takes precedence when both are defined.
- `rs doc` requires the optional `@rspress/core` dependency.

## rstack.config.ts

Rstack CLI loads `rstack.config.{ts,js,mts,mjs}` by default.

Register config with `define.*`:

```ts
import { define } from 'rstack';

define.app({
  // Rsbuild config for `rs dev`, `rs build`, and `rs preview`
});

define.test({
  // Rstest config for `rs test`
});
```

- `define.app(config)`: Rsbuild config for `rs dev`, `rs build`, and `rs preview`. Docs: https://rsbuild.rs/config/
- `define.lib(config)`: Rslib config for `rs lib`; Docs: https://rslib.rs/config/
- `define.doc(config)`: Rspress config for `rs doc`; Docs: https://rspress.rs/api/config/config-basic
- `define.test(config)`: Rstest config for `rs test`; Docs: https://rstest.rs/config/
- `define.lint(config)`: Rslint config for `rs lint`; Docs: https://rslint.rs/config/
- `define.staged(config)`: lint-staged config for `rs staged`; accepts `Record<string, string | string[]>`.

### Lazy Configuration

Prefer async functions with dynamic imports for dependencies. Avoid top-level sync imports of heavy dependencies in `rstack.config.ts`.

```ts
import { define } from 'rstack';

define.app(async () => {
  const { pluginReact } = await import('@rsbuild/plugin-react');
  return {
    plugins: [pluginReact()],
  };
});
```

## Import Paths

Prefer Rstack-exported paths:

| Instead of                | Prefer                   |
| ------------------------- | ------------------------ |
| `@rsbuild/core`           | `rstack/app`             |
| `@rslib/core`             | `rstack/lib`             |
| `@rstest/core`            | `rstack/test`            |
| `@rslint/core`            | `rstack/lint`            |
| `@rsbuild/core/types`     | `rstack/types`           |
| `@rslib/core/types`       | `rstack/types`           |
| `@rstest/core/globals`    | `rstack/test/globals`    |
| `@rstest/core/importMeta` | `rstack/test/importMeta` |

## Git Hooks

Use [`rs setup`](https://rstack.rs/guide/cli/setup) for project-local Git hooks, commonly with `rs staged` in a `pre-commit` hook.
