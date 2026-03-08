use memchr;
use rspack_core::{DependencyLocation, DependencyRange, RealDependencyLocation, SourcePosition};

/// Advances source positions incrementally to compute dependency locations efficiently.
/// This optimization reduces repeated source scans when processing dependencies
/// in increasing source order (common for import statements).
#[derive(Debug, Default)]
pub struct DependencyLocationAdvancer {
  last_range: Option<DependencyRange>,
  last_location: Option<DependencyLocation>,
  last_start_pos: Option<SourcePosition>,
}

impl DependencyLocationAdvancer {
  pub fn new() -> Self {
    Self::default()
  }

  /// Advance a source position from one byte offset to another, counting newlines and UTF-16 columns.
  /// Optimized with ASCII fast-paths and SIMD reverse searching.
  fn advance_pos(
    source: &str,
    from_off: usize,
    from_pos: SourcePosition,
    to_off: usize,
  ) -> Option<SourcePosition> {
    if to_off < from_off || to_off > source.len() {
      return None;
    }

    let segment = &source[from_off..to_off];
    let bytes = segment.as_bytes();

    let (newline_count, last_newline_idx) = memchr::memchr_iter(b'\n', bytes)
      .enumerate()
      .last()
      .map_or((0, None), |(count, idx)| (count + 1, Some(idx)));

    if let Some(last_idx) = last_newline_idx {
      let line = from_pos.line + newline_count;
      let after_newline = &segment[last_idx + 1..];
      let column = after_newline.encode_utf16().count() + 1; // 1-based column
      Some(SourcePosition { line, column })
    } else {
      let column_advance = segment.encode_utf16().count();
      Some(SourcePosition {
        line: from_pos.line,
        column: from_pos.column + column_advance,
      })
    }
  }

  /// Compute dependency location for a range, using cached results for incremental calculation.
  pub fn compute_dependency_location(
    &mut self,
    source: &str,
    range: DependencyRange,
  ) -> Option<DependencyLocation> {
    let start = range.start as usize;
    let end = range.end as usize;

    // Fast path: same range as last time
    if Some(range) == self.last_range {
      return self.last_location.clone();
    }

    // Determine the base point for calculation
    let (base_offset, base_pos) = if let (Some(last_range), Some(last_start_pos)) =
      (self.last_range, self.last_start_pos)
      && start >= last_range.start as usize
    {
      // Incremental path: Use the previously cached position
      (last_range.start as usize, last_start_pos)
    } else {
      // Fallback path: Start calculating from the beginning of the file (1-based)
      (0, SourcePosition { line: 1, column: 1 })
    };

    // Uniformly use advance_pos for both incremental and fallback calculations
    let result = (|| {
      let start_pos = Self::advance_pos(source, base_offset, base_pos, start)?;
      let end_pos = Self::advance_pos(source, start, start_pos, end)?;

      // Uniformly construct the Location return value
      if start_pos.line == end_pos.line && start_pos.column == end_pos.column {
        Some(DependencyLocation::Real(RealDependencyLocation::new(
          start_pos, None,
        )))
      } else {
        Some(DependencyLocation::Real(RealDependencyLocation::new(
          start_pos,
          Some(end_pos),
        )))
      }
    })();

    // Update cache
    if let Some(loc) = &result {
      self.last_range = Some(range);
      self.last_location = Some(loc.clone());

      if let DependencyLocation::Real(real_loc) = loc {
        self.last_start_pos = Some(real_loc.start);
      }
    }

    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_same_range_cache() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "import a from './a';\nimport b from './b';";
    let range = DependencyRange::new(0, 20);

    let loc1 = cache.compute_dependency_location(source, range);
    let loc2 = cache.compute_dependency_location(source, range);

    // Compare by converting to string representation
    assert_eq!(
      loc1.as_ref().map(|l| l.to_string()),
      loc2.as_ref().map(|l| l.to_string())
    );
    assert!(loc1.is_some());
  }

  #[test]
  fn test_incremental_calculation() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "import a from './a';\nimport b from './b';\nimport c from './c';";

    // First location
    let range1 = DependencyRange::new(0, 20);
    let loc1 = cache.compute_dependency_location(source, range1).unwrap();

    // Second location (after first, should use incremental calculation)
    let range2 = DependencyRange::new(21, 41);
    let loc2 = cache.compute_dependency_location(source, range2).unwrap();

    // Verify locations are correct
    if let DependencyLocation::Real(real1) = &loc1 {
      assert_eq!(real1.start.line, 1);
    }
    if let DependencyLocation::Real(real2) = &loc2 {
      assert_eq!(real2.start.line, 2);
    }
  }

  #[test]
  fn test_fallback_to_full_calculation() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "import a from './a';\nimport b from './b';\nimport c from './c';";

    // First location
    let range1 = DependencyRange::new(21, 41);
    let loc1 = cache.compute_dependency_location(source, range1);

    // Second location (before first, should fallback to full calculation)
    let range2 = DependencyRange::new(0, 20);
    let loc2 = cache.compute_dependency_location(source, range2);

    // Both should be valid
    assert!(loc1.is_some());
    assert!(loc2.is_some());

    if let Some(DependencyLocation::Real(real1)) = &loc1 {
      assert_eq!(real1.start.line, 2);
    }
    if let Some(DependencyLocation::Real(real2)) = &loc2 {
      assert_eq!(real2.start.line, 1);
    }
  }

  #[test]
  fn test_advance_pos_same_line() {
    let source = "hello world";
    let from_pos = SourcePosition { line: 1, column: 1 };

    // Advance from position 0 to 5 ("hello")
    let result = DependencyLocationAdvancer::advance_pos(source, 0, from_pos, 5);
    assert!(result.is_some());
    let pos = result.unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 6); // 1 + 5 UTF-16 units
  }

  #[test]
  fn test_advance_pos_multiline() {
    let source = "hello\nworld";
    let from_pos = SourcePosition { line: 1, column: 6 };

    // Advance from position 5 (after "hello") to 11 (end of "world")
    let result = DependencyLocationAdvancer::advance_pos(source, 5, from_pos, 11);
    assert!(result.is_some());
    let pos = result.unwrap();
    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 6); // "world" = 5 UTF-16 units, column 6 (1-based)
  }

  #[test]
  fn test_advance_pos_utf8_multibyte() {
    let source = "你好世界";
    let from_pos = SourcePosition { line: 1, column: 1 };

    // Advance from 0 to 6 bytes (first two characters "你好")
    let result = DependencyLocationAdvancer::advance_pos(source, 0, from_pos, 6);
    assert!(result.is_some());
    let pos = result.unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 3); // 1 + 2 UTF-16 units
  }

  #[test]
  fn test_advance_pos_emoji() {
    let source = "hello😀world";
    let from_pos = SourcePosition { line: 1, column: 1 };

    // Advance from 0 to 9 bytes (includes emoji)
    let result = DependencyLocationAdvancer::advance_pos(source, 0, from_pos, 9);
    assert!(result.is_some());
    let pos = result.unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 8); // "hello" = 5, emoji = 2 UTF-16 units, so 7 (1-based)
  }

  #[test]
  fn test_multiple_ranges() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "import a from './a';\nimport b from './b';\nimport c from './c';";

    // Test multiple ranges to ensure cache works correctly
    let ranges = vec![
      DependencyRange::new(0, 20),
      DependencyRange::new(21, 41),
      DependencyRange::new(42, 62),
    ];

    for range in ranges {
      let result = cache.compute_dependency_location(source, range);
      assert!(
        result.is_some(),
        "Should compute location for range {range:?}",
      );

      // Verify the result has correct structure
      if let Some(DependencyLocation::Real(real_loc)) = &result {
        assert!(real_loc.start.line > 0, "Line should be 1-based");
        assert!(real_loc.start.column > 0, "Column should be 1-based");
      }
    }
  }

  #[test]
  fn test_empty_source() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "";
    let range = DependencyRange::new(0, 0);

    let result = cache.compute_dependency_location(source, range);
    assert!(result.is_some());
  }

  #[test]
  fn test_single_line() {
    let mut cache = DependencyLocationAdvancer::new();
    let source = "single line";
    let range = DependencyRange::new(0, 11);

    let result = cache.compute_dependency_location(source, range).unwrap();
    if let DependencyLocation::Real(real) = result {
      assert_eq!(real.start.line, 1);
      assert_eq!(real.end.as_ref().map(|e| e.line), Some(1));
    }
  }
}
