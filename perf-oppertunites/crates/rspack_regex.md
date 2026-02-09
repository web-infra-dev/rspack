# rspack_regex

## Role
Regex utilities used throughout the codebase.

## Profiling relevance
- Indirectly visible when regex-based matching is used in hot paths.
- Hot in plugins that apply patterns to many modules.

## Perf opportunities
- Precompile regexes and reuse across calls.
- Avoid regex usage in hot paths; prefer manual parsers.
- Cache match results for repeated patterns when possible.

## Suggested experiments
- Benchmark regex-heavy rules (e.g., ignore patterns) with and without caching.
- Measure regex compilation counts in large builds.

## Code pointers
- `crates/rspack_regex/**`
