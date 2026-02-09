# rspack_binding_build

## Role
Binding build script for Node.js integration.

## Perf opportunities
- Not runtime hot; ensure build scripts are not invoked in production builds.
- Avoid expensive build-time checks unless required.

## Code pointers
- `crates/rspack_binding_build/**`
