# rspack_browser

## Role
Browser/WASM support for Rspack.

## Profiling relevance
- Not visible in react-10k; relevant for browser/WASM builds.
- Costs scale with WASM initialization and interop.

## Perf opportunities
- Avoid loading browser shims in Node builds.
- Keep WASM initialization minimal; cache module instantiation.
- Reduce serialization overhead when transferring data to WASM.

## Key functions/structs to inspect
- `oneshot` helpers (oneshot.rs) for browser initialization.
- Panic handling shims (panic.rs).
- WASM entrypoints in `lib.rs`.

## Suggested experiments
- Profile browser builds with large projects and measure WASM init time.
- Compare cached vs fresh WASM instantiation.

## Code pointers
- `crates/rspack_browser/Cargo.toml`
- `crates/rspack_browser/src/lib.rs`
- `crates/rspack_browser/src/oneshot.rs`
- `crates/rspack_browser/src/panic.rs`
- `crates/rspack_browser/**`
