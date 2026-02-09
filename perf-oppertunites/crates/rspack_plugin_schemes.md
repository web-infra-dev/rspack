# rspack_plugin_schemes

## Role
Scheme handling (e.g., `file:`, `data:`, `http:`) for module resolution.

## Perf opportunities
- Cache scheme dispatch results to avoid repeated parsing.
- Avoid converting resources to strings when scheme is already known.
- Batch remote scheme IO to reduce per-resource overhead.

## Code pointers
- `crates/rspack_plugin_schemes/**`
