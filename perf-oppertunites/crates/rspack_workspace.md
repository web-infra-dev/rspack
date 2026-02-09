# rspack_workspace

## Role
Workspace-level utilities shared across crates (configuration glue, helper types).

## Perf opportunities
- Reuse normalized workspace paths/config structures instead of recomputing per pass.
- Avoid cloning large configuration structs; use `Arc` or borrowed references.
- Prefer `Cow<str>` for string helpers to reduce allocations in hot helpers.

## Code pointers
- `crates/rspack_workspace/**`
