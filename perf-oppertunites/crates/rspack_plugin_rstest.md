# rspack_plugin_rstest

## Role
Rstest integration for testing workflows.

## Profiling relevance
- Not in runtime hot path; only active in test workflows.
- Ensure no overhead in production builds.

## Perf opportunities
- Ensure plugin work is gated to test runs only.
- Avoid expensive file scanning when rstest is disabled.
- Reuse parsed configuration across runs.

## Suggested experiments
- Verify rstest hooks are disabled in production mode.
- Measure config parse time with large test suites.

## Code pointers
- `crates/rspack_plugin_rstest/**`
