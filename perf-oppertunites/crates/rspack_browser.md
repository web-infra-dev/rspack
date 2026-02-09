# rspack_browser

## Role
Browser/WASM support for Rspack.

## Perf opportunities
- Avoid loading browser shims in Node builds.
- Keep WASM initialization minimal; cache module instantiation.
- Reduce serialization overhead when transferring data to WASM.

## Code pointers
- `crates/rspack_browser/**`
