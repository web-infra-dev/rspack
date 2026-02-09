# rspack_plugin_entry

## Role
Entry plugin to register and manage initial entries.

## Profiling relevance
- Not visible in react-10k; hot when many entries are configured.
- Costs scale with entry dependency resolution.

## Perf opportunities
- Cache computed entry dependency lists.
- Avoid reallocating entry vectors on incremental builds.
- Short-circuit when entries are unchanged.

## Suggested experiments
- Profile multi-entry builds to measure entry setup time.
- Compare cached vs recomputed entry dependency lists.

## Code pointers
- `crates/rspack_plugin_entry/**`
