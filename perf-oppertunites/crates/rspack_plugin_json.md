# rspack_plugin_json

## Role
JSON module support.

## Profiling relevance
- Not visible in react-10k; hot when many JSON modules are bundled.
- Costs scale with JSON parse and stringify operations.

## Perf opportunities
- Avoid converting JSON to strings unnecessarily; keep as bytes or AST.
- Cache parsed JSON for unchanged modules.
- Short-circuit JSON parsing for small inline JSON modules.

## Suggested experiments
- Profile builds with many JSON modules to measure parse overhead.
- Compare cache hit rates for unchanged JSON files.

## Code pointers
- `crates/rspack_plugin_json/Cargo.toml`
- `crates/rspack_plugin_json/src/lib.rs`
- `crates/rspack_plugin_json/**`
