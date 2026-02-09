# rspack_binding_build

## Role
Binding build script for Node.js integration.

## Profiling relevance
- Build-time only; no runtime impact.
- Ensure build scripts are not invoked unnecessarily.

## Perf opportunities
- Not runtime hot; ensure build scripts are not invoked in production builds.
- Avoid expensive build-time checks unless required.

## Suggested experiments
- Validate build script execution frequency in CI.

## Code pointers
- `crates/rspack_binding_build/Cargo.toml`
- `crates/rspack_binding_build/src/lib.rs`
- `crates/rspack_binding_build/**`
