# rspack_location

## Role
Location tracking utilities (source positions and dependency locations).

## Profiling relevance
- Not visible in react-10k; used when diagnostics or source maps require locations.
- Costs scale with number of tracked locations.

## Perf opportunities
- Avoid building location data in release builds unless needed.
- Reuse location structs for repeated source positions.
- Skip expensive formatting in hot paths.

## Suggested experiments
- Measure cost of location tracking with source maps enabled.
- Compare builds with and without detailed location tracking.

## Code pointers
- `crates/rspack_location/Cargo.toml`
- `crates/rspack_location/src/lib.rs`
- `crates/rspack_location/**`
