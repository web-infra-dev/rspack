# Rsbuild Config Debug Tool

Use this tool only when the failure hypothesis depends on the actual Rsbuild/Rspack configuration used by the downstream case.

## Trigger

Run this tool when any of these are true:

- The user mentions `DEBUG=rsbuild`.
- The suspected Rspack PR changes behavior that is controlled by a Rsbuild/Rspack config option, plugin, loader, target, devtool, cache mode, SSR mode, or environment.
- The failing case is built through an Rsbuild-based tool such as Rsbuild, Rslib, Rstest, Modern.js, or Rspress, and you need to decide whether the failed case is actually related to the current PR.

Do not run this tool just because the suite is Rsbuild-based. Use it when config evidence can change the attribution.

## Process

1. Identify the narrowest downstream fixture or package that contains the failing test.
2. Run its build command with Rsbuild debug enabled:

   ```bash
   DEBUG=rsbuild npm run build
   ```

   Use the package manager and script that the downstream repo actually uses, such as `DEBUG=rsbuild pnpm build`, `DEBUG=rsbuild pnpm test <case>`, or the fixture-specific build command.

3. Inspect generated config files under `dist/.rsbuild/`, commonly:

   ```text
   dist/.rsbuild/rsbuild.config.mjs
   dist/.rsbuild/rspack.config.web.mjs
   dist/.rsbuild/rspack.config.node.mjs
   ```

4. Search the generated configs for the option, plugin, loader, target, mode, or environment that links the PR diff to the failing case.
5. Compare the active config with the current PR diff and the failure log.

## Decision Rules

- If the generated config does not enable the option or path required by the hypothesis, the current PR is likely only a surface pivot.
- If the generated config enables the relevant option and the failure signature maps to the PR diff, return to Phase 2 and explain the mechanism.
- If the generated config is inconclusive, report the config evidence gap instead of forcing attribution.

## Output

```text
Config check: <active | inactive | inconclusive>
Command: <DEBUG=rsbuild command>
Generated config files: <paths>
Relevant config: <short summary>
Attribution impact: <supports candidate | weakens candidate | rules out candidate | inconclusive>
```
