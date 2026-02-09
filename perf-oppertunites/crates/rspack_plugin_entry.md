# rspack_plugin_entry

## Role
Entry plugin to register and manage initial entries.

## Perf opportunities
- Cache computed entry dependency lists.
- Avoid reallocating entry vectors on incremental builds.
- Short-circuit when entries are unchanged.

## Code pointers
- `crates/rspack_plugin_entry/**`
