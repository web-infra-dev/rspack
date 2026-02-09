# rspack_plugin_copy

## Role
Copy assets or files to output directory.

## Perf opportunities
- Batch file copy operations to reduce IO overhead.
- Avoid hashing or reading files when unchanged.
- Skip copy pass when configuration has no patterns.

## Code pointers
- `crates/rspack_plugin_copy/**`
