# rspack_plugin_banner

## Role
Banner injection into emitted assets.

## Perf opportunities
- Cache banner strings and reuse across assets.
- Avoid inserting banners when disabled or empty.
- Use `String::with_capacity` for concatenations.

## Code pointers
- `crates/rspack_plugin_banner/**`
