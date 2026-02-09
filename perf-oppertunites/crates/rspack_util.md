# rspack_util

## Role
Utility helpers used across the codebase (path, string, tracing helpers).

## Perf opportunities
- Ensure hot helpers avoid allocation; prefer `Cow<str>`/borrowed slices.
- Cache frequently used conversions (path normalization, hashing).
- Avoid regex use in hot paths; replace with lightweight parsers.

## Code pointers
- `crates/rspack_util/**`
