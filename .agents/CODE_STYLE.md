# Code style facts

This file records repository style facts that agents should obey. Prefer the
checked-in configuration files over generic language conventions.

## Formatting commands

- Rust: `pnpm run format:rs` runs `cargo fmt --all`.
- JavaScript, TypeScript, Markdown, and other Prettier-managed files:
  `pnpm run format:js` runs `prettier . --write && heading-case --write`.
- TOML: `pnpm run format:toml` runs `taplo format`.
- CI checks are `pnpm run format-ci:js` and `pnpm run format-ci:toml`.

Use `corepack pnpm` if the bundled pnpm version does not match
`packageManager`.

## Repository configuration

- `.editorconfig` sets spaces with size 2 for normal files.
- `.prettierrc` sets `singleQuote: true`.
- `rustfmt.toml` sets Rust edition/style edition 2024 and `tab_spaces = 2`.
- `package.json` defines `lint:rs` as `cargo check --workspace --all-targets --locked`.
- `package.json` defines `lint:js` as `rslint --fix` and `lint-ci:js` as `rslint`.

Do not override these facts with generic TypeScript or Rust style defaults.

## Rust rules worth remembering

- Use `rspack_error::Result<T>` for fallible Rspack operations.
- Avoid `block_on` in async contexts.
- Avoid allocation-heavy string methods flagged by `clippy.toml`; prefer
  `cow_utils::CowUtils` helpers where the lint points there.
- Follow the concurrency boundary in `AGENTS.md`: `rayon` for CPU-bound
  synchronous parallel work, `rspack_parallel` for async orchestration.

## JavaScript and TypeScript rules worth remembering

- Follow Prettier output; this repo uses single quotes.
- Keep public API types precise and exported from the appropriate package entry.
- Do not use `any` for public API shape unless compatibility requires it and the
  trade-off is documented.

## Markdown rules

- `heading-case` enforces sentence-case headings.
- Keep agent notes factual and linked to source paths. Avoid duplicating
  glossary definitions that belong in `CONTEXT.md`.
