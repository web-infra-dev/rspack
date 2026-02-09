# rspack_plugin_rstest

## Role
Rstest integration for testing workflows.

## Perf opportunities
- Ensure plugin work is gated to test runs only.
- Avoid expensive file scanning when rstest is disabled.
- Reuse parsed configuration across runs.

## Code pointers
- `crates/rspack_plugin_rstest/**`
