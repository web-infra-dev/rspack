# node_binding (rspack_node)

## Role
Node.js binding entrypoint for Rspack.

## Perf opportunities
- Batch JS↔Rust calls to reduce NAPI overhead.
- Prefer zero-copy buffers for sources/assets.
- Avoid cloning large structures when marshaling to JS.

## Code pointers
- `crates/node_binding/**`
