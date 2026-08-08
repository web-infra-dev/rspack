# rspack_ids

## Role
ID generation and management for modules/chunks.

## Profiling relevance
- Not visible in react-10k; hot when many modules/chunks are created.
- Costs scale with hashing and ID assignment strategy.

## Perf opportunities
- Cache derived IDs to avoid repeated hashing.
- Use incremental ID assignment where possible.
- Avoid string allocations when IDs are numeric.

## Key functions/structs to inspect
- Deterministic ID plugins (`deterministic_module_ids_plugin.rs`).
- Named/natural ID plugins (`named_module_ids_plugin.rs`, `natural_module_ids_plugin.rs`).
- `assign_ids` helpers in `id_helpers.rs`.

## Suggested experiments
- Measure ID assignment time in large module graphs.
- Compare hashed vs incremental ID strategies.

## Code pointers
- `crates/rspack_ids/Cargo.toml`
- `crates/rspack_ids/src/lib.rs`
- `crates/rspack_ids/src/id_helpers.rs`
- `crates/rspack_ids/src/deterministic_module_ids_plugin.rs`
- `crates/rspack_ids/src/named_module_ids_plugin.rs`
- `crates/rspack_ids/**`
