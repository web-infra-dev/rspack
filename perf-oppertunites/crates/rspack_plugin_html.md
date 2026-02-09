# rspack_plugin_html

## Role
HTML generation plugin (template rendering and asset injection).

## Profiling relevance
- Not visible in react-10k samples; can be significant when many HTML pages are generated.
- Costs scale with template size and asset list length.

## Perf opportunities
- Cache rendered templates by configuration.
- Avoid repeated asset list string building when unchanged.
- Use streaming template rendering for large HTML outputs.

## Suggested experiments
- Profile multi-page HTML builds and measure render time.
- Compare cached vs non-cached template rendering.

## Code pointers
- `crates/rspack_plugin_html/**`
