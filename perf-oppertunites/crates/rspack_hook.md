# rspack_hook

## Role
Hook system definitions and macros.

## Profiling relevance
- Not explicitly visible in flat samples; hook dispatch overhead appears in plugin-heavy builds.
- Costs scale with number of taps and hook invocations.

## Perf opportunities
- Add fast paths when no taps are registered.
- Avoid building tap lists for empty hooks.
- Reduce allocations in hook argument preparation.

## Suggested experiments
- Measure hook dispatch counts and time for plugin-heavy builds.
- Compare fast-path hook checks vs current behavior.

## Code pointers
- `crates/rspack_hook/Cargo.toml`
- `crates/rspack_hook/src/lib.rs`
- `crates/rspack_hook/**`
