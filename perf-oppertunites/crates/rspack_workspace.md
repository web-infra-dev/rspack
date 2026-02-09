# rspack_workspace

## Role
Workspace-level utilities shared across crates (configuration glue, helper types).

## Profiling relevance
- Not directly visible in the react-10k perf samples; primarily affects setup and shared helpers.
- Becomes relevant when configuration normalization runs frequently (multi-compiler, watch).

## Perf opportunities
- Reuse normalized workspace paths/config structures instead of recomputing per pass.
- Avoid cloning large configuration structs; use `Arc` or borrowed references.
- Prefer `Cow<str>` for string helpers to reduce allocations in hot helpers.

## Suggested experiments
- Measure config normalization time with and without caching (multi-compiler config).
- Track allocations during `CompilerOptions` normalization to validate wins.

## Code pointers
- `crates/rspack_workspace/**`
