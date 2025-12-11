# rspack_storage_compare

A tool for comparing two rspack cache directories (`.cache` files) to verify they contain identical data.

## Purpose

This crate is designed to support portable cache testing in rspack. It enables comparison of cache files generated on different platforms (Windows, Linux, macOS) or in different project paths to ensure cache portability.

## Features

- Compare cache directories across different platforms
- Detect differences in:
  - Scope availability
  - Key-value pairs within scopes
  - Data content
- Detailed difference reporting
- Exit codes for CI integration (0 = identical, 1 = different)

## Usage

### As a Binary

```bash
# Compare two cache directories
cargo run --bin rspack_storage_compare -- \
  --cache1 /path/to/cache1 \
  --cache2 /path/to/cache2

# Verbose output
cargo run --bin rspack_storage_compare -- \
  --cache1 /path/to/cache1 \
  --cache2 /path/to/cache2 \
  --verbose

# Or build and run directly
cargo build --release -p rspack_storage_compare
./target/release/rspack_storage_compare --cache1 /path/to/cache1 --cache2 /path/to/cache2
```

### As a Library

```rust
use rspack_storage_compare::compare_storage_dirs;
use rspack_paths::Utf8PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let path1 = Utf8PathBuf::from("/path/to/cache1/.cache");
  let path2 = Utf8PathBuf::from("/path/to/cache2/.cache");

  let result = compare_storage_dirs(path1, path2).await?;

  if result.is_equal {
    println!("Caches are identical!");
  } else {
    println!("Differences found:");
    for diff in result.differences {
      println!("  {:?}", diff);
    }
  }

  Ok(())
}
```

## CI Integration

This tool is designed to be used in CI pipelines to verify cache portability:

1. **Generate caches** on different platforms (Linux, Windows, macOS)
2. **Upload cache artifacts** from each platform
3. **Download all caches** in a comparison job
4. **Run comparison** using this tool
5. **Fail the build** if caches differ (exit code 1)

### Example CI Workflow

```yaml
jobs:
  generate-cache-linux:
    runs-on: ubuntu-latest
    steps:
      - name: Build and cache
        run: npm run build
      - name: Upload cache
        uses: actions/upload-artifact@v3
        with:
          name: cache-linux
          path: .cache

  generate-cache-windows:
    runs-on: windows-latest
    steps:
      - name: Build and cache
        run: npm run build
      - name: Upload cache
        uses: actions/upload-artifact@v3
        with:
          name: cache-windows
          path: .cache

  compare-caches:
    needs: [generate-cache-linux, generate-cache-windows]
    runs-on: ubuntu-latest
    steps:
      - name: Download Linux cache
        uses: actions/download-artifact@v3
        with:
          name: cache-linux
          path: cache-linux
      - name: Download Windows cache
        uses: actions/download-artifact@v3
        with:
          name: cache-windows
          path: cache-windows
      - name: Compare caches
        run: |
          cargo run --bin rspack_storage_compare -- \
            --cache1 cache-linux \
            --cache2 cache-windows \
            --verbose
```

## Design

The tool works by:

1. **Reading both cache directories** using `PackStorage` from `rspack_storage`
2. **Enumerating all scopes** in each cache using the `Storage::scopes()` method
3. **Loading key-value pairs** for each scope
4. **Comparing**:
   - Scope names
   - Key presence
   - Value equality (byte-by-byte)
5. **Reporting differences** with detailed information

### Implementation Details

- Uses the new `Storage::scopes()` method added to `rspack_storage` to get all available scopes
- The `scopes()` method reads from the root metadata file, ensuring consistency with what was actually saved
- Avoids manual directory scanning, which could miss scopes or include invalid directories

### Architecture

```
rspack_storage_compare/
├── src/
│   ├── lib.rs              # Public API
│   ├── storage_reader.rs   # Read cache directories
│   ├── comparator.rs       # Comparison logic
│   └── main.rs             # CLI entry point
├── tests/
│   └── integration_test.rs # Integration tests
└── Cargo.toml
```

## Related

- RFC: [Portable Cache #12218](https://github.com/web-infra-dev/rspack/discussions/12218)
- `rspack_storage` - The underlying storage implementation
