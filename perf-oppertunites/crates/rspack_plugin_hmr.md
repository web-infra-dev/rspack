# rspack_plugin_hmr

## Role
Hot Module Replacement runtime and hooks.

## Perf opportunities
- Gate HMR logic strictly to dev/watch builds.
- Cache runtime template fragments by HMR mode.
- Avoid per-module string formatting for HMR metadata.

## Code pointers
- `crates/rspack_plugin_hmr/**`
