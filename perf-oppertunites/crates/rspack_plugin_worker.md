# rspack_plugin_worker

## Role
Worker plugin for handling worker entry points and chunking.

## Perf opportunities
- Cache resolved worker entry modules to avoid repeated resolution.
- Reuse generated worker runtime templates across chunks.
- Avoid string concatenations in per-module worker wrappers.

## Code pointers
- `crates/rspack_plugin_worker/**`
