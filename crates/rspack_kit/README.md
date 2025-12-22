# rspack_kit

A comprehensive toolkit for debugging and testing rspack internals.

## Purpose

This crate provides a collection of utilities for rspack developers and advanced users to inspect, validate, and test various internal components of rspack, with a focus on cache functionality and build artifacts.

## Commands

### `compare` - Compare Cache Directories

Compare two cache directories to verify they contain identical data. This is essential for:

- Testing cache portability across platforms (Windows, Linux, macOS)
- Validating cache consistency across different build environments
- CI/CD cache verification workflows

**Features:**

- Deep comparison of module graphs and dependencies
- Build artifact validation
- Metadata and strategy verification
- Detailed difference reporting with hierarchical context
- Exit codes for CI integration (0 = identical, 1 = different)

**Usage:**

```bash
rspack_kit compare /path/to/cache1 /path/to/cache2
```

## Installation & Usage

### As a Binary

```bash
# Build the tool
cargo build --release -p rspack_kit

# Run commands
./target/release/rspack_kit compare /path/to/cache1 /path/to/cache2

# Show help
./target/release/rspack_kit --help
./target/release/rspack_kit compare --help
```

### As a Library

```rust
use rspack_kit::compare_storage_dirs;
use rspack_paths::Utf8PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let path1 = Utf8PathBuf::from("/path/to/cache1/.cache");
  let path2 = Utf8PathBuf::from("/path/to/cache2/.cache");

  compare_storage_dirs(path1, path2).await?;

  println!("Caches are identical!");
  Ok(())
}
```

## CI Integration Example

Verify cache portability across platforms in GitHub Actions:

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
          cargo run --bin rspack_kit -- \
            compare cache-linux cache-windows
```

## Implementation Details

### Compare Command

The comparison tool provides deep validation of cache equivalence:

**Comparison Strategy:**

1. Load both storage directories and enumerate version directories
2. Compare all scopes (snapshot, build_dependencies, occasion/meta, occasion/make)
3. Perform scope-specific deep comparison:
   - **Snapshot** - File path to version strategy mapping
   - **Build dependencies** - Dependency resolution strategies
   - **Meta** - Build metadata (skips regenerated fields)
   - **Make artifacts** - Module graphs with two-pass validation:
     - First pass: Compare dependencies and build DependencyId mapping
     - Second pass: Verify module relations using the mapping
     - Final pass: Compare BuildInfo with mapped dependency references

**Key Design Decisions:**

- Uses DependencyId mapping to handle non-deterministic IDs across builds
- Leverages `rspack_cacheable` serialization for comprehensive field comparison
- Provides hierarchical error context for easy debugging

### Architecture

```
rspack_kit/
├── src/
│   ├── lib.rs                    # Public API
│   ├── main.rs                   # CLI with subcommands
│   ├── debug_info.rs             # Hierarchical context tracking
│   ├── utils.rs                  # Shared utilities
│   └── compare/
│       ├── mod.rs                # Compare entry point
│       ├── snapshot.rs           # Snapshot comparison
│       ├── build_dependencies.rs # Build deps comparison
│       └── occasion/
│           ├── meta.rs           # Meta comparison
│           └── make.rs           # Make artifacts comparison
└── Cargo.toml
```

## Future Enhancements

The toolkit is designed for extensibility. Potential future commands include:

- `rspack_kit inspect <cache>` - Interactive cache inspector
- `rspack_kit validate <cache>` - Integrity validation
- `rspack_kit stats <cache>` - Cache statistics and analysis
- `rspack_kit dump <cache>` - Export cache contents in readable format
- `rspack_kit diff <cache1> <cache2>` - Human-readable diff output
- `rspack_kit benchmark <cache>` - Performance benchmarking
- `rspack_kit migrate <cache>` - Version migration utilities
- `rspack_kit verify <build>` - Build artifact verification
- `rspack_kit trace <module>` - Dependency tracing and visualization

## Related

- RFC: [Portable Cache #12218](https://github.com/web-infra-dev/rspack/discussions/12218)
- `rspack_storage` - The underlying storage implementation
- `rspack_core` - Core rspack functionality
