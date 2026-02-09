# rspack_regex

## Role
Regex utilities used throughout the codebase.

## Perf opportunities
- Precompile regexes and reuse across calls.
- Avoid regex usage in hot paths; prefer manual parsers.
- Cache match results for repeated patterns when possible.

## Code pointers
- `crates/rspack_regex/**`
