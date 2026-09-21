use memchr;
use rspack_core::{DependencyLocation, DependencyRange, RealDependencyLocation, SourcePosition};

/// For pure-ASCII slices (the common case, e.g. minified vendor sources) the
/// UTF-16 code-unit count equals the byte length, and `is_ascii` is a
/// SIMD-accelerated byte scan — much cheaper than decoding every char.
#[inline]
fn utf16_len(s: &str) -> usize {
  if s.is_ascii() {
    s.len()
  } else {
    s.encode_utf16().count()
  }
}

/// Advances source positions incrementally to compute dependency locations efficiently.
///
/// Two pieces of state cooperate:
/// - a lazily built, incrementally extended index of line-start byte offsets.
///   It resolves the line of any offset with a binary search, so dependencies
///   that are visited out of source order (or after a jump) never force a
///   rescan from the beginning of the module;
/// - the previously computed position, which keeps the column computation of
///   subsequent dependencies proportional to the gap since the last one
///   instead of the whole line.
#[derive(Debug, Default)]
pub struct DependencyLocationAdvancer {
  last_range: Option<DependencyRange>,
  last_location: Option<DependencyLocation>,
  last_start_pos: Option<SourcePosition>,
  /// Byte offset of the first character of every line, from the beginning of
  /// the source up to the byte at index indexed_upto. Always starts with 0.
  line_starts: Vec<u32>,
  /// Number of leading source bytes that have been scanned for newlines.
  indexed_upto: usize,
  /// Identity of the source the index was built for. 0 means "no source yet"
  /// (a str pointer is never null).
  indexed_source_ptr: usize,
  indexed_source_len: usize,
}

impl DependencyLocationAdvancer {
  pub fn new() -> Self {
    Self::default()
  }

  /// Prepare the caches for the source and make sure every line start at or
  /// before max_off is present in the index.
  #[inline]
  fn prepare_source(&mut self, source: &str, max_off: usize) {
    let ptr = source.as_ptr() as usize;
    let len = source.len();

    if ptr != self.indexed_source_ptr || len != self.indexed_source_len {
      // Defensive: the advancer is owned by one parser and therefore one
      // source, but never let a stale index or stale positions leak into a
      // different source.
      self.last_range = None;
      self.last_location = None;
      self.last_start_pos = None;
      self.line_starts.clear();
      self.line_starts.push(0);
      self.indexed_upto = 0;
      self.indexed_source_ptr = ptr;
      self.indexed_source_len = len;
      self.line_starts.reserve(len / 32);
    }

    // Offsets fit in u32 for every realistic module (AST spans are u32);
    // huge sources simply keep the scanning fallback.
    if max_off > self.indexed_upto && u32::try_from(len).is_ok() {
      for newline in memchr::memchr_iter(b'\n', &source.as_bytes()[self.indexed_upto..max_off]) {
        self
          .line_starts
          .push((self.indexed_upto + newline + 1) as u32);
      }
      self.indexed_upto = max_off;
    }
  }

  /// Whether the newline index currently covers the offset for the source.
  #[inline]
  fn index_covers(&self, source: &str, off: usize) -> bool {
    self.indexed_source_ptr == source.as_ptr() as usize
      && self.indexed_source_len == source.len()
      && off <= self.indexed_upto
  }

  /// Position of an offset resolved through the newline index.
  ///
  /// The offset must be covered by the index; the caller guarantees it, and the
  /// u32 casts are safe because the index is only built for sources no larger
  /// than u32::MAX.
  #[inline]
  fn position_from_index(source: &str, line_starts: &[u32], off: usize) -> SourcePosition {
    // Line starts begin with 0, so this is at least 1 and maps 1:1 to the
    // 1-based line number.
    let line = line_starts.partition_point(|&start| start as usize <= off);
    let line_start = line_starts[line - 1] as usize;
    SourcePosition {
      line: line as u32,
      column: (utf16_len(&source[line_start..off]) + 1) as u32,
    }
  }

  /// Position of an offset, advancing from a known position at base_off.
  #[inline]
  fn position(
    &self,
    source: &str,
    off: usize,
    base_off: usize,
    base_pos: SourcePosition,
  ) -> Option<SourcePosition> {
    if !self.index_covers(source, off) {
      return Self::advance_pos(source, base_off, base_pos, off);
    }

    let index_pos = Self::position_from_index(source, &self.line_starts, off);
    if index_pos.line != base_pos.line {
      return Some(index_pos);
    }

    // Same line as the cached position: only the gap needs to be measured.
    debug_assert!(base_off <= off);
    Some(SourcePosition {
      line: base_pos.line,
      column: base_pos
        .column
        .checked_add(u32::try_from(utf16_len(&source[base_off..off])).ok()?)?,
    })
  }

  /// Advance a source position from one byte offset to another, counting newlines and UTF-16 columns.
  /// Optimized with ASCII fast-paths and SIMD newline searching.
  ///
  /// Only used when the newline index cannot cover the requested offset (i.e.
  /// sources larger than u32::MAX); the regular path is Self::position.
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
      let line = from_pos
        .line
        .checked_add(u32::try_from(newline_count).ok()?)?;
      let after_newline = &segment[last_idx + 1..];
      let column = u32::try_from(utf16_len(after_newline) + 1).ok()?; // 1-based column
      Some(SourcePosition { line, column })
    } else {
      let column_advance = u32::try_from(utf16_len(segment)).ok()?;
      Some(SourcePosition {
        line: from_pos.line,
        column: from_pos.column.checked_add(column_advance)?,
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

    // Mirror advance_pos, which rejects backwards and out-of-bounds offsets.
    if start > end || end > source.len() {
      return None;
    }

    self.prepare_source(source, end);

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

    // Uniformly use the indexed position for both incremental and fallback calculations
    let result = (|| {
      let start_pos = self.position(source, start, base_offset, base_pos)?;
      let end_pos = self.position(source, end, start, start_pos)?;

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

  /// Independent implementation of the position rules the advancer must
  /// reproduce: 1-based lines and columns, with columns counted in UTF-16
  /// code units (V8 compatible).
  fn reference_pos(source: &str, off: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut line_start = 0usize;
    for (idx, byte) in source.bytes().enumerate() {
      if idx >= off {
        break;
      }
      if byte == b'\n' {
        line += 1;
        line_start = idx + 1;
      }
    }
    let column = (utf16_len(&source[line_start..off]) + 1) as u32;
    (line, column)
  }

  fn assert_locations_match_reference(
    cache: &mut DependencyLocationAdvancer,
    source: &str,
    ranges: &[DependencyRange],
  ) {
    for range in ranges {
      let location = cache
        .compute_dependency_location(source, *range)
        .unwrap_or_else(|| panic!("expected a location for {range:?}"));
      let DependencyLocation::Real(real) = location else {
        panic!("expected a real location for {range:?}");
      };

      let expected_start = reference_pos(source, range.start as usize);
      assert_eq!(
        (real.start.line, real.start.column),
        expected_start,
        "start of {range:?}"
      );

      let expected_end = reference_pos(source, range.end as usize);
      if expected_end == expected_start {
        assert!(real.end.is_none(), "expected no end for {range:?}");
      } else {
        let end = real
          .end
          .unwrap_or_else(|| panic!("expected an end for {range:?}"));
        assert_eq!((end.line, end.column), expected_end, "end of {range:?}");
      }
    }
  }

  /// Every possible range between UTF-8 boundaries (spans always are on
  /// boundaries), so the advancer is exercised at line boundaries, on empty
  /// ranges, at the end of the file and - depending on the order the ranges are
  /// passed in - on both forward and backward movements.
  fn all_ranges(source: &str) -> Vec<DependencyRange> {
    let points: Vec<u32> = (0..=source.len())
      .filter(|offset| source.is_char_boundary(*offset))
      .map(|offset| offset as u32)
      .collect();
    let mut ranges = Vec::new();
    for (index, start) in points.iter().enumerate() {
      for end in &points[index..] {
        ranges.push(DependencyRange::new(*start, *end));
      }
    }
    ranges
  }

  const SOURCE_WITH_UNICODE: &str = "import a from './a';\n\n// 注释 comment 😀\nexport const b = 你好;\r\nimport c from './c';\nimport d from './d';\n";

  #[test]
  fn test_index_matches_reference_in_source_order() {
    let mut cache = DependencyLocationAdvancer::new();
    assert_locations_match_reference(
      &mut cache,
      SOURCE_WITH_UNICODE,
      &all_ranges(SOURCE_WITH_UNICODE),
    );
  }

  #[test]
  fn test_index_matches_reference_in_reverse_order() {
    let mut cache = DependencyLocationAdvancer::new();
    let mut ranges = all_ranges(SOURCE_WITH_UNICODE);
    ranges.reverse();
    assert_locations_match_reference(&mut cache, SOURCE_WITH_UNICODE, &ranges);
  }

  #[test]
  fn test_index_matches_reference_in_shuffled_order() {
    // Deterministic shuffle (LCG) to mix forward jumps, backward jumps and
    // repeated ranges.
    let mut ranges = all_ranges(SOURCE_WITH_UNICODE);
    let mut state = 0x2545_f491_4f6c_dd1du64;
    for i in (1..ranges.len()).rev() {
      state = state
        .wrapping_mul(0x5851_f42d_4c95_7f2d)
        .wrapping_add(0x1405_7b7e_f767_814f);
      ranges.swap(i, (state >> 33) as usize % (i + 1));
    }
    let mut cache = DependencyLocationAdvancer::new();
    assert_locations_match_reference(&mut cache, SOURCE_WITH_UNICODE, &ranges);
  }

  #[test]
  fn test_index_matches_reference_on_minified_single_line() {
    let source = "var a=1;var b=2;var c=3;";
    let mut cache = DependencyLocationAdvancer::new();
    assert_locations_match_reference(&mut cache, source, &all_ranges(source));
  }

  #[test]
  fn test_index_survives_source_switch() {
    let mut cache = DependencyLocationAdvancer::new();
    let first = "import a from './a';\nimport b from './b';\n";
    let second = "\n\nconst x = 你好;\nconst y = 2;\n";
    assert_locations_match_reference(&mut cache, first, &all_ranges(first));
    assert_locations_match_reference(&mut cache, second, &all_ranges(second));
    assert_locations_match_reference(&mut cache, first, &all_ranges(first));
  }

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
