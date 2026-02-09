# rspack_plugin_json

## Role
JSON module support.

## Perf opportunities
- Avoid converting JSON to strings unnecessarily; keep as bytes or AST.
- Cache parsed JSON for unchanged modules.
- Short-circuit JSON parsing for small inline JSON modules.

## Code pointers
- `crates/rspack_plugin_json/**`
