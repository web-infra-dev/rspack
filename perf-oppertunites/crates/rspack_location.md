# rspack_location

## Role
Location tracking utilities (source positions and dependency locations).

## Perf opportunities
- Avoid building location data in release builds unless needed.
- Reuse location structs for repeated source positions.
- Skip expensive formatting in hot paths.

## Code pointers
- `crates/rspack_location/**`
