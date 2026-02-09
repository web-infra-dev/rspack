# rspack_plugin_lazy_compilation

## Role
Lazy compilation support for deferred module building.

## Perf opportunities
- Ensure lazy compilation checks are low overhead when disabled.
- Cache module eligibility results to avoid repeated checks.
- Avoid runtime template regeneration for identical lazy entries.

## Code pointers
- `crates/rspack_plugin_lazy_compilation/**`
