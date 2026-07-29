---
name: migrate-to-rstack-cli
description: Use when migrating projects from standalone Rsbuild, Rslib, Rstest, Rslint, Rspress, lint-staged, husky or simple-git-hooks tooling to the unified `rstack` package, `rs` commands, and `rstack.config.*`.
---

# Migrate to Rstack CLI

Rstack CLI is the `rstack` package, exposed through the `rs` binaries. It provides one CLI, one config file, and a consistent workflow for the Rstack JavaScript toolchain.

## Tool References

Read every matching reference before editing. Load only the tools present in the project.

- `@rsbuild/core`, `rsbuild.config.*`, `rsbuild` commands, or Rsbuild types: [rsbuild.mdx](references/rsbuild.mdx)
- `@rslib/core`, `rslib.config.*`, `rslib` commands, or Rslib types: [rslib.mdx](references/rslib.mdx)
- `@rstest/core`, `@rstest/adapter-*`, `rstest.config.*`, `rstest` commands, or test imports: [rstest.mdx](references/rstest.mdx)
- `@rslint/core`, `rslint.config.*`, `rslint` commands, or lint imports: [rslint.mdx](references/rslint.mdx)
- `@rspress/core`, `rspress.config.*`, `rspress` commands, themes, or plugins: [rspress.mdx](references/rspress.mdx)
- `lint-staged`, `nano-staged`, their configs: [lint-staged.mdx](references/lint-staged.mdx)
- `husky`, `.husky/`, `package.json#husky`, `simple-git-hooks`, `.simple-git-hooks.*`, or Git hook installer scripts: [git-hooks.mdx](references/git-hooks.mdx)

## Workflow

1. Inspect manifests, workspace catalogs, lock files, scripts, standalone configs, Git hooks, TypeScript `types`, and source imports.
2. Read the matching references and inventory behavior that must survive: config functions, CLI arguments, plugins, presets, adapters, custom config paths, and chained commands.
3. Check the latest `rstack` version and inspect its Node.js engine and underlying tool versions. Resolve plugin and adapter peer ranges first; upgrade incompatible extensions or stop when no compatible version exists. Add `rstack` using the repository's existing package manager and version convention, usually as a development dependency.
4. If a matching reference uses a `define.*` registration, create `rstack.config.ts` and move the standalone configuration into it.
5. Rewrite commands and imports as directed by the references.
6. Search again for old direct imports, binaries, config paths, manifest entries, and type references. Remove only entries with no remaining direct or runtime use and no unresolved peer compatibility requirement.
7. Delete a standalone config only after its behavior is represented in `rstack.config.*`.
8. Refresh the lockfile with the repository's package manager. Confirm the expected tool version changes and resolve peer dependency warnings.
9. Run the repository's existing migrated scripts and required checks. Compare generated artifacts or runtime behavior where relevant.

Underlying Rsbuild, Rslib, Rstest, and Rslint packages remain transitive dependencies of `rstack`. Do not require their names to disappear from the lockfile; require obsolete direct manifest entries and imports to disappear.

## Configuration Rules

Use one of the default names: `rstack.config.ts`, `.js`, `.mts`, or `.mjs`.

Use `rs -c <path>` or `rs --config <path>` only for a custom path.

```ts
import { define } from 'rstack';

define.app({
  // Rsbuild config
});

define.test({
  // Rstest config
});
```

Prefer async config functions and dynamic imports for runtime plugins and presets:

```ts
define.lint(async () => {
  const { js, ts } = await import('rstack/lint');
  return [js.configs.recommended, ts.configs.recommended];
});
```

Rstack loads TypeScript configs as native ESM. Preserve runtime-resolvable file extensions, replace CommonJS globals such as `__dirname`.
