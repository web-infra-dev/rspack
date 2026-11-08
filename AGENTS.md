# Rspack

Rspack is a Rust-based JavaScript bundler with a webpack-compatible API.

## Build and validation

Before testing changed code, build the artifacts those tests consume using the [development commands](.agents/DEVELOPMENT.md#build).

## Adding tests

- **Rust:** Do not add new Rust tests in ordinary changes. New cases belong in dedicated test crates; do not add inline `#[test]` functions or crate-local unit tests.
- **JavaScript:** Reuse existing runners and add cases under `tests/rspack-test/{type}Cases/`. Do not add top-level or suite-level `test.js` runners. Harness-required files inside a case, such as `hookCases/**/test.js`, are allowed.

## Pull requests

- Follow [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md).
- Update relevant English and Chinese docs when public behavior or APIs change.

## References

- [Concurrency](.agents/ARCHITECTURE.md#parallel-processing): read before changing parallel execution or task scheduling.
- [Cache and Incremental](.agents/CACHE_AND_INCREMENTAL.md): read before changing either cache backend or shared cache dependencies.
- [Rspack Sources](.agents/RSPACK_SOURCES.md): read before changing or heavily using `crates/rspack_sources`.
- [Binding](.agents/BINDING.md): read when changing Rust/JavaScript ownership, lifetimes, or hook bridging.
- [Rust cloning](.agents/CODE_STYLE.md#cloning) and [error handling](.agents/CODE_STYLE.md#error-handling): read the relevant rules before adding `Clone` implementations or changing Rust error handling.
- [Architecture](.agents/ARCHITECTURE.md) and [project layout](website/docs/en/contribute/development/project.md): compilation flow and subsystem locations.
- [API design](.agents/API_DESIGN.md): public contracts and webpack compatibility.
- [Code style](.agents/CODE_STYLE.md), [common patterns](.agents/COMMON_PATTERNS.md), and [glossary](.agents/GLOSSARY.md): conventions and terminology.
- [Testing](website/docs/en/contribute/development/testing.mdx) and [debugging](website/docs/en/contribute/development/debugging.mdx): harness and debugger details.
