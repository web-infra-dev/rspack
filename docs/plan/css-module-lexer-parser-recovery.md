# CSS Modules lexer recovery migration plan

## Goal

Move the raw-source CSS Modules and ICSS syntax recovery currently implemented in
`rspack_plugin_css::parser` into `/home/jinzhixin/rstack/css-module-lexer`.
Rspack should consume structured lexer dependencies and warnings without rescanning
the original CSS source.

The final Rspack dependency must remain pinned to an exact commit from
`https://github.com/intellild/css-module-lexer.git`.

## Scope and responsibility boundary

### css-module-lexer owns

- Parsing and validating `@value`, including warning for declarations such as
  `@value test;` whose value is missing.
- Recognizing comment-separated mode pseudos such as
  `:local/** comment **/.class`, `:local/** comment **/#id`,
  `:global/** comment **/.class`, and `:local/** comment **/{`.
- Emitting the required `Replace`, `LocalClass`, and `LocalId` dependencies and a
  structured `MissingWhitespace` warning for those mode pseudos.
- Returning exact, comment-free ICSS import request slices from `@value ... from`
  declarations. Request extraction must be based on the current token positions,
  not a global source search.
- Classifying bare `@import name;` as `ICSSImportUrl` with its name and ranges.
  This structured dependency is the only syntax information Rspack should need.

### Rspack owns

- Mapping lexer warnings into Rspack diagnostics.
- Resolving an `ICSSImportUrl` name through ICSS definitions and filesystem/module
  request rules.
- Emitting `Expected URL` when a structured `ICSSImportUrl` remains unresolved.
  This is semantic validation because the lexer cannot know the ICSS symbol table.
- Creating import, compose, export, and re-export dependencies and connecting them
  to the module graph.
- Normalizing and resolving module requests.

## Rejected alternatives

- Keep the Rspack source scanners as a fallback: rejected because they duplicate
  lexer state, use brittle global `source.find()` recovery, and can generate
  dependencies inconsistent with the lexer token stream.
- Add a second CSS parser or an AST API: rejected as unnecessarily broad. The
  existing `Dependency` and `WarningKind` APIs can represent the required output.
- Make the lexer always warn for `@import name;`: rejected because this syntax is
  valid when `name` resolves through an ICSS definition. Rspack must make that
  semantic decision from the structured `ICSSImportUrl`.

## TDD implementation steps

### 1. Add failing css-module-lexer tests

File: `/home/jinzhixin/rstack/css-module-lexer/tests/test.rs`

- Add a test for `@value test;` asserting:
  - one `Unexpected("Broken '@value' at-rule")` warning over the whole at-rule;
  - existing export and replace dependencies remain stable.
- Add focused tests for the four comment-separated mode pseudo forms, asserting:
  - exactly one `MissingWhitespace { surrounding: "trailing" }` warning;
  - the mode marker/comment range is removed exactly once;
  - local class/id dependencies are emitted with `explicit: true`;
  - global selectors do not create local dependencies.
- Add comment-rich `@value ... from` tests asserting `ICSSImportFrom.path` contains
  only the quoted or identifier request, without surrounding comments.
- Keep the existing `ICSSImportUrl` tests and add a comment/spacing case if needed
  to prove that `name_range` remains exact.

Run the focused tests before implementation and record that the new assertions fail
for the expected reasons.

### 2. Implement lexer behavior

File: `/home/jinzhixin/rstack/css-module-lexer/src/dependencies.rs`

- In `lex_value_at_rule`, preserve the existing empty-local-name warning and add a
  warning only for a bare name whose value is missing. Colon declarations with an
  empty or comment-only value must preserve the historical behavior.
- Keep the main balanced-stack and nested-selector behavior unchanged. Broadening
  that state machine causes mixed global/local nested selectors that webpack leaves
  untouched to be parsed and exported.
- Run a focused recovery pass inside `collect_dependencies` for comment/no-space
  `:local` and `:global` forms. The pass owns the source scan inside the lexer,
  deduplicates dependencies and warnings, and never exposes syntax scanning to
  Rspack.
- Normalize recovered `Replace` ranges to the mode marker only so CSS comments
  remain in generated output, matching webpack snapshots.
- Preserve the historical parsing of comment-heavy `@value` names, values, and
  import items. Recover only the final `from` request from aligned source/masked
  token positions; do not make unrelated comment-heavy declarations effective.
- Preserve the existing `ICSSImportUrl` API unless a small additive field is
  required; avoid breaking unrelated consumers.

The targeted recovery pass is intentionally narrower than a second CSS parser. It
exists to preserve established visitor behavior while moving Rspack's former
compatibility scan into the lexer that owns these dependency types.

Validation in lexer repository:

```text
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

Commit and push the lexer changes to `origin/codex/bugfix/css-module-lexer`, then
capture the exact full commit SHA.

### 3. Remove Rspack raw-source recovery

File:
`crates/rspack_plugin_css/src/parser_and_generator/parser.rs`

- Remove:
  - `scan_css_identifier_end`
  - `add_mode_replace_dependency`
  - `strip_css_comments`
  - `is_invalid_bare_import`
  - `last_quoted_string`
  - `add_missing_value_at_rule_warnings`
  - `add_missing_whitespace_mode_dependencies`
  - `add_missing_whitespace_mode_warning`
  - `recover_icss_import_request_from_source`
- Restore direct consumption of `collect_dependencies`.
- Handle `ICSSImportUrl` from its structured fields:
  - resolve the lexer-provided name;
  - create the import when it resolves to a usable request;
  - otherwise emit the existing `Expected URL` diagnostic using its lexer range.
- Reduce `resolve_icss_import_request` to semantic ICSS/filesystem resolution and
  quote normalization only.
- Keep dependency creation and warning-to-diagnostic mapping in Rspack.

### 4. Pin the new lexer commit

Files:

- `Cargo.toml`
- `Cargo.lock`

Replace the old revision
`4619f65d150844b3a60ad538e59f404d6466c1fe` with the exact full SHA pushed from
`origin/codex/bugfix/css-module-lexer`. Regenerate `Cargo.lock` through Cargo and
verify that both the `rev` query and resolved source fragment are exact.

### 5. Rspack acceptance tests

Build first because Rust code changed:

```text
pnpm run build:binding:dev
```

Run the focused compatibility tests from `tests/rspack-test`:

```text
pnpm exec rstest --project base -t configCases/css/css-modules
pnpm exec rstest --project base -t serialCases/css/css-modules-no-space
```

Then run the repository-required broad checks, skipping sandbox-hanging storage and
native watcher suites as instructed by `AGENTS.md`:

```text
pnpm run test:rs
pnpm run test:unit
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## Acceptance criteria

- Rspack parser contains no raw-source CSS Modules syntax scanner introduced by
  the current branch.
- Lexer unit tests cover every migrated recovery behavior.
- Existing webpack-derived CSS Modules snapshots and warning order stay unchanged.
- `css-module-lexer` is committed and pushed to the intended origin branch.
- Rspack `Cargo.toml` and `Cargo.lock` pin the same exact new lexer commit SHA.
- Focused CSS Modules tests, Rust tests, unit tests, clippy, formatting, and diff
  checks pass, with documented exclusions only for storage/native watcher tests.
