# rspack_tools

A toolkit for debugging and testing rspack internals.

## Commands

### `compare` - Compare cache in directories

Compare the cache content of two directories to verify they contain identical data.

**Usage:**

```bash
rspack_tools compare /path/to/cache1 /path/to/cache2
```

### `bench-diff` - Compare hotpath JSON benchmark reports

Compare two `layer=hotpath` JSON reports and print the overall elapsed time and per-function metric diffs.

**Usage:**

```bash
rspack_tools bench-diff /path/to/before.json /path/to/after.json
```
