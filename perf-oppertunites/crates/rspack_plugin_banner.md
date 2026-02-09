# rspack_plugin_banner

## Role
Banner injection into emitted assets.

## Profiling relevance
- Not visible in react-10k; hot when banners are enabled for many assets.
- Costs scale with asset count and banner size.

## Perf opportunities
- Cache banner strings and reuse across assets.
- Avoid inserting banners when disabled or empty.
- Use `String::with_capacity` for concatenations.

## Suggested experiments
- Measure banner injection overhead on large asset outputs.
- Compare cached banner string reuse across rebuilds.

## Code pointers
- `crates/rspack_plugin_banner/**`
