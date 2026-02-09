# rspack

## Role
Top-level crate entrypoint for Rspack.

## Perf opportunities
- Keep entrypoint thin to avoid pulling extra dependencies.
- Avoid unnecessary initialization work in the common path.
- Use lazy initialization for global state.

## Code pointers
- `crates/rspack/**`
