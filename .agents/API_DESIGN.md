# API design constraints

Use this file when changing public Rspack APIs, configuration shape, plugin
interfaces, loader behavior, or webpack compatibility.

## Compatibility first

Rspack's public API should match webpack where users or ecosystem packages
reasonably expect compatibility.

- Match webpack configuration names, defaults, hook names, and runtime behavior
  unless there is an explicit reason not to.
- If behavior differs from webpack, document the gap in code comments, tests, or
  user-facing docs as appropriate.
- Prefer additive API changes. Treat removed options, renamed options, signature
  changes, and default-behavior changes as breaking unless proven otherwise.

## Performance trade-offs

Performance can override exact webpack behavior only when the trade-off is
intentional and visible.

- Keep the compatibility fallback or migration path where practical.
- Add tests that lock the chosen behavior.
- Mention surprising incompatibilities in docs or diagnostics.

## Type surface

Public JavaScript APIs should remain typed and discoverable.

- Export public types from the package entry points.
- Prefer precise TypeScript types over `any`.
- Keep config object shapes compatible with existing webpack-style usage.
- Do not expose Rust implementation details as public API vocabulary.

## Errors and diagnostics

Errors should be actionable.

- Include relevant file paths, requests, config option names, and source
  locations when available.
- Prefer messages that explain what failed and how the user can fix it.
- Preserve existing diagnostics snapshots when behavior is intentionally
  unchanged.

## Tests and docs

Public API changes need matching validation.

- Add or update `tests/rspack-test/` cases for compatibility-sensitive behavior.
- Update `website/docs/en/api/` when user-facing API behavior changes.
- For plugin and loader compatibility, prefer tests that resemble real webpack
  usage rather than isolated unit-only coverage.

## Deprecation and breaking changes

Follow semver expectations from `AGENTS.md`.

- `fix:` should be patch-compatible.
- `feat:` should be minor-compatible.
- Breaking changes need `!` or `BREAKING CHANGE:` and migration guidance.
- Deprecations should include a replacement path before removal.
