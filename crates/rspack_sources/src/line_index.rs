#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
struct WideChar {
  utf16_start: u32,
  byte_start: u32,
}

/// UTF-16 columns for a whole source. Only non-ASCII lines have character indexes.
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct LineIndex<'text> {
  #[cfg_attr(allocative, allocative(skip))]
  text: &'text str,
  line_starts: Vec<u32>,
  // Empty when the whole source is ASCII. Otherwise indexed by line number.
  line_wide_chars: Vec<Option<Box<[WideChar]>>>,
}

#[derive(Clone, Copy)]
pub struct IndexedLine<'text, 'index> {
  pub line: &'text str,
  wide: Option<&'index [WideChar]>,
}

#[derive(Default)]
struct ScanState {
  line_start: usize,
  extra_bytes: usize,
  chars: Vec<WideChar>,
}

impl<'text> LineIndex<'text> {
  pub fn new(text: &'text str) -> Self {
    Self::with_known_ascii(text, false)
  }

  pub fn with_known_ascii(text: &'text str, is_known_ascii: bool) -> Self {
    debug_assert!(!is_known_ascii || text.is_ascii());
    let mut index = Self {
      text,
      line_starts: if text.is_empty() { Vec::new() } else { vec![0] },
      line_wide_chars: Vec::new(),
    };
    if is_known_ascii {
      for newline in memchr::memchr_iter(b'\n', text.as_bytes()) {
        let next = newline + 1;
        if next < text.len() {
          index.line_starts.push(next as u32);
        }
      }
    } else {
      index.scan();
    }
    index
  }

  pub fn line_count(&self) -> usize {
    self.line_starts.len()
  }

  #[allow(unsafe_code)]
  pub fn get(&self, line: usize) -> Option<IndexedLine<'text, '_>> {
    if line >= self.line_count() {
      return None;
    }
    // SAFETY: The line number was checked against the line count above.
    Some(unsafe { self.get_unchecked(line) })
  }

  /// # Safety
  /// `line` must be less than `self.line_count()`.
  #[allow(unsafe_code)]
  pub(crate) unsafe fn get_unchecked(&self, line: usize) -> IndexedLine<'text, '_> {
    // SAFETY: The caller guarantees that `line` is a valid line number.
    let start = unsafe { *self.line_starts.get_unchecked(line) } as usize;
    let end = if line + 1 < self.line_count() {
      // SAFETY: The next line number was checked against the line count.
      (unsafe { *self.line_starts.get_unchecked(line + 1) }) as usize
    } else {
      self.text.len()
    };
    let text: &'text str = self.text;
    // SAFETY: Line starts are recorded in order at 0 or immediately after an
    // ASCII newline, so both offsets are ordered UTF-8 boundaries in `text`.
    let line_text = unsafe { text.get_unchecked(start..end) };
    let wide = if self.line_wide_chars.is_empty() {
      None
    } else {
      // SAFETY: Once populated, the wide-character vector has one slot per line.
      unsafe { self.line_wide_chars.get_unchecked(line) }.as_deref()
    };
    IndexedLine {
      line: line_text,
      wide,
    }
  }

  fn record_newline(&mut self, byte_start: usize, state: &mut ScanState) {
    self.finish_line(state);
    let next = byte_start + 1;
    if next < self.text.len() {
      self.line_starts.push(next as u32);
      if !self.line_wide_chars.is_empty() {
        self.line_wide_chars.push(None);
      }
    }
    state.line_start = next;
  }

  fn finish_line(&mut self, state: &mut ScanState) {
    if !state.chars.is_empty() {
      if self.line_wide_chars.is_empty() {
        self
          .line_wide_chars
          .resize_with(self.line_starts.len(), || None);
      }
      self.line_wide_chars[self.line_starts.len() - 1] =
        Some(std::mem::take(&mut state.chars).into_boxed_slice());
    }
    state.extra_bytes = 0;
  }

  /// The cursor may pass `limit` when a UTF-8 character crosses a SIMD block.
  fn scan_scalar_to(&mut self, cursor: &mut usize, limit: usize, state: &mut ScanState) {
    while *cursor < limit {
      let first_byte = self.text.as_bytes()[*cursor];
      if first_byte == b'\n' {
        self.record_newline(*cursor, state);
        *cursor += 1;
      } else if first_byte < 0x80 {
        *cursor += 1;
      } else {
        let byte_len = first_byte.leading_ones() as usize;
        debug_assert!((2..=4).contains(&byte_len));
        let byte_start = *cursor - state.line_start;
        let utf16_start = byte_start - state.extra_bytes;
        let wide_len = if byte_len == 4 { 2 } else { 1 };
        let extra_bytes = byte_len - wide_len;
        state.chars.push(WideChar {
          utf16_start: utf16_start as u32,
          byte_start: byte_start as u32,
        });
        state.extra_bytes += extra_bytes;
        *cursor += byte_len;
      }
    }
  }

  fn scan_generic(&mut self) {
    let mut cursor = 0;
    let mut state = ScanState::default();
    self.scan_scalar_to(&mut cursor, self.text.len(), &mut state);
    self.finish_line(&mut state);
  }

  // Adapted from rust-analyzer's line-index scanner: ASCII blocks record
  // newlines in parallel; blocks containing UTF-8 are decoded one character
  // at a time. The other architectures use the scalar scanner.
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  fn scan(&mut self) {
    if is_x86_feature_detected!("sse2") {
      // SAFETY: Runtime detection guarantees SSE2 support.
      unsafe { self.scan_sse2() }
    } else {
      self.scan_generic();
    }
  }

  #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
  fn scan(&mut self) {
    if std::arch::is_aarch64_feature_detected!("neon") {
      // SAFETY: Runtime detection guarantees NEON support.
      unsafe { self.scan_neon() }
    } else {
      self.scan_generic();
    }
  }

  #[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_endian = "little")
  )))]
  fn scan(&mut self) {
    self.scan_generic();
  }

  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  #[target_feature(enable = "sse2")]
  #[allow(unsafe_code)]
  unsafe fn scan_sse2(&mut self) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 16;
    let ptr = self.text.as_ptr();
    let len = self.text.len();
    let mut cursor = 0;
    let mut state = ScanState::default();
    while cursor + CHUNK_SIZE <= len {
      // SAFETY: The loop condition keeps the unaligned load inside `text`.
      let chunk = unsafe { _mm_loadu_si128(ptr.add(cursor).cast::<__m128i>()) };
      if _mm_movemask_epi8(chunk) == 0 {
        let matches = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
        let mut mask = _mm_movemask_epi8(matches) as u32;
        while mask != 0 {
          let bit = mask.trailing_zeros() as usize;
          self.record_newline(cursor + bit, &mut state);
          mask &= mask - 1;
        }
        cursor += CHUNK_SIZE;
      } else {
        let limit = cursor + CHUNK_SIZE;
        self.scan_scalar_to(&mut cursor, limit, &mut state);
      }
    }
    self.scan_scalar_to(&mut cursor, len, &mut state);
    self.finish_line(&mut state);
  }

  #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
  #[target_feature(enable = "neon")]
  #[allow(unsafe_code)]
  unsafe fn neon_mask(v: std::arch::aarch64::uint8x16_t) -> u64 {
    use std::arch::aarch64::*;

    let nibbles = vshrn_n_u16(vreinterpretq_u16_u8(v), 4);
    vget_lane_u64(vreinterpret_u64_u8(nibbles), 0)
  }

  #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
  #[target_feature(enable = "neon")]
  #[allow(unsafe_code)]
  unsafe fn scan_neon(&mut self) {
    use std::arch::aarch64::*;

    const CHUNK_SIZE: usize = 16;
    let ptr = self.text.as_ptr();
    let len = self.text.len();
    let mut cursor = 0;
    let mut state = ScanState::default();
    let newline = vdupq_n_u8(b'\n');
    while cursor + CHUNK_SIZE <= len {
      // SAFETY: The loop condition keeps the load inside `text`.
      let chunk = unsafe { vld1q_u8(ptr.add(cursor)) };
      let non_ascii = vcltzq_s8(vreinterpretq_s8_u8(chunk));
      // SAFETY: This function has the same NEON target feature.
      if unsafe { Self::neon_mask(non_ascii) } == 0 {
        let matches = vceqq_u8(chunk, newline);
        // SAFETY: This function has the same NEON target feature.
        let mut mask = unsafe { Self::neon_mask(matches) };
        while mask != 0 {
          let bit = (mask.trailing_zeros() / 4) as usize;
          self.record_newline(cursor + bit, &mut state);
          mask &= !(0xF_u64 << (bit * 4));
        }
        cursor += CHUNK_SIZE;
      } else {
        let limit = cursor + CHUNK_SIZE;
        self.scan_scalar_to(&mut cursor, limit, &mut state);
      }
    }
    self.scan_scalar_to(&mut cursor, len, &mut state);
    self.finish_line(&mut state);
  }
}

impl<'text> IndexedLine<'text, '_> {
  /// Slice by UTF-16 columns, rounding a position inside a surrogate pair down
  /// to the start of its UTF-8 character.
  #[inline(always)]
  #[allow(unsafe_code)]
  pub fn substring(&self, start_utf16_index: usize, end_utf16_index: usize) -> &'text str {
    if end_utf16_index <= start_utf16_index {
      return "";
    }

    let Some(wide) = self.wide else {
      let start = start_utf16_index.min(self.line.len());
      let end = end_utf16_index.min(self.line.len());
      // SAFETY: A line without indexed characters is ASCII, and the clamped
      // offsets are ordered UTF-8 boundaries.
      return unsafe { self.line.get_unchecked(start..end) };
    };

    let to_byte = |index: usize| {
      if index >= self.line.len() {
        return self.line.len();
      }
      let position = wide.partition_point(|ch| ch.utf16_start as usize <= index);
      let Some(ch) = position.checked_sub(1).and_then(|i| wide.get(i)) else {
        return index;
      };
      let byte_start = ch.byte_start as usize;
      let byte_len = self.line.as_bytes()[byte_start].leading_ones() as usize;
      let wide_len = if byte_len == 4 { 2 } else { 1 };
      let after = index - ch.utf16_start as usize;
      if after < wide_len {
        byte_start
      } else {
        let extra_bytes = byte_start - ch.utf16_start as usize + byte_len - wide_len;
        index.saturating_add(extra_bytes).min(self.line.len())
      }
    };

    let start = to_byte(start_utf16_index);
    let end = to_byte(end_utf16_index);
    // SAFETY: Indexed starts are UTF-8 boundaries. Positions within a
    // surrogate pair round down; later positions land in ASCII runs or at
    // another indexed character, and both offsets clamp to the line length.
    unsafe { self.line.get_unchecked(start..end) }
  }
}

#[cfg(test)]
mod tests {
  use super::LineIndex;

  #[test]
  fn test_substring() {
    let index = LineIndex::new("foobar");
    assert_eq!(index.get(0).unwrap().substring(0, 3), "foo");
  }

  #[test]
  fn test_out_of_bounds() {
    let index = LineIndex::new("foobar");
    assert_eq!(index.get(0).unwrap().substring(0, 10), "foobar");
    assert_eq!(index.get(0).unwrap().substring(6, 10), "");
  }

  #[test]
  fn test_start_less_than_end() {
    let index = LineIndex::new("foobar");
    assert_eq!(index.get(0).unwrap().substring(3, 2), "");
  }

  #[test]
  fn test_start_and_end_equal() {
    let index = LineIndex::new("foobar");
    assert_eq!(index.get(0).unwrap().substring(3, 3), "");
  }

  #[test]
  fn test_multiple_byte_characters() {
    let index = LineIndex::new("🙈🙉🙊🐒");
    assert_eq!(index.get(0).unwrap().substring(2, 4), "🙉");
  }
}
