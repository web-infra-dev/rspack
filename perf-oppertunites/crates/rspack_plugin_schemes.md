# rspack_plugin_schemes

## Role
Scheme handling (e.g., `file:`, `data:`, `http:`) for module resolution.

## Profiling relevance
- Not visible in react-10k (mostly `file:`).
- Hot when using `http:` or custom schemes with many requests.

## Perf opportunities
- Cache scheme dispatch results to avoid repeated parsing.
- Avoid converting resources to strings when scheme is already known.
- Batch remote scheme IO to reduce per-resource overhead.

## Suggested experiments
- Profile a build with many `http:` modules to measure scheme handler costs.
- Evaluate cache hit rates for repeated scheme requests.

## Code pointers
- `crates/rspack_plugin_schemes/**`
