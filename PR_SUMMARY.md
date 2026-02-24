# Performance: Optimize DependencyLocation Computation with Incremental Caching

## Summary

This PR introduces a significant performance optimization for `DependencyLocation` computation in the JavaScript plugin. The optimization reduces redundant source file scans by implementing incremental position calculation with caching, particularly beneficial when processing multiple dependencies in the same file (e.g., multiple import statements).

## Problem

Previously, computing `DependencyLocation` for each dependency required scanning the entire source file from the beginning to calculate line and column numbers. This resulted in:
- **O(n×m) complexity** for n dependencies in a file of length m
- Redundant UTF-16 encoding calculations
- Repeated newline counting operations
- Performance degradation in files with many import/export statements

## Solution

### Core Changes

1. **New `DependencyLocationAdvancer` struct**
   - Implements incremental position calculation using cached results
   - Maintains last computed range, location, and start position
   - Uses `memchr` for efficient byte-level newline searching
   - Optimized with single-pass iteration and fast-path for same-range lookups

2. **Incremental Calculation Algorithm**
   - **Cache hit**: Returns cached result if the same range is requested
   - **Incremental path**: When new range starts after the last one, advances from the cached position instead of scanning from the beginning
   - **Fallback path**: When new range starts before the last one, falls back to full calculation from file start
   - **Unified logic**: Both paths use the same `advance_pos` helper for consistency

3. **API Refactoring**
   - Changed `to_dependency_location` parameter from `Span` to `DependencyRange` for better type safety
   - Moved location computation out of dependency constructors
   - All dependency constructors now accept pre-computed `DependencyLocation`
   - Updated all parser plugins to use the new cached API

### Performance Optimizations

- **Fast-path caching**: Same range lookups return immediately
- **Incremental advancement**: Only scans the segment between last position and new target
- **Efficient newline counting**: Uses `memchr` with SIMD optimizations
- **Reduced allocations**: Single-pass iteration with minimal string operations
- **UTF-16 optimization**: Only encodes segments that need column calculation

## Implementation Details

### Key Components

- **`DependencyLocationAdvancer`**: Core caching and calculation logic
  - `compute_dependency_location()`: Main entry point with caching
  - `advance_pos()`: Incremental position advancement helper

- **Updated `JavascriptParser`**: 
  - Embeds `DependencyLocationAdvancer` instance
  - Delegates location computation to the advancer

- **Dependency Constructors**: 
  - Accept `loc: Option<DependencyLocation>` directly
  - Removed internal `range.to_loc(source)` calls

- **Parser Plugins**: 
  - All updated to compute location before creating dependencies
  - Use `parser.to_dependency_location(range)` API

### Code Quality

- ✅ Comprehensive unit tests covering:
  - Cache hit scenarios
  - Incremental calculation paths
  - Fallback to full calculation
  - Edge cases (empty source, single line, UTF-8, emoji)
  - Position advancement logic
- ✅ Follows Rust naming conventions (`compute_*` for mutating operations)
- ✅ No breaking changes to public APIs (internal refactoring)

## Impact

### Performance Benefits

- **Reduced complexity**: From O(n×m) to O(m + n×k) where k is average segment length
- **Faster parsing**: Especially noticeable in files with many imports/exports
- **Lower memory overhead**: Caches only last result (last-only cache strategy)

### Files Changed

- **32 files modified**
- **509 insertions, 209 deletions**
- Core changes in:
  - `location_advancer.rs` (new file, ~290 lines)
  - `parser/mod.rs` (integration)
  - All dependency constructors (API updates)
  - All parser plugins (call site updates)

## Breaking Changes

⚠️ **API Change**: `JavascriptParser::to_dependency_location()` now accepts `DependencyRange` instead of `Span`. This is an internal API change that affects parser plugins but maintains backward compatibility for external users.

## Testing

- ✅ All existing tests pass
- ✅ New comprehensive test suite for `DependencyLocationAdvancer`
- ✅ Tests cover incremental paths, fallback paths, and edge cases
- ✅ Validates UTF-8, emoji, and multibyte character handling

## Migration Guide

For plugin developers:
- Update calls from `parser.to_dependency_location(span)` to `parser.to_dependency_location(DependencyRange::from(span))`
- Or use `DependencyRange::new(start, end)` directly if you have byte offsets

## Related Issues

Fixes performance issues when parsing files with many dependencies.
