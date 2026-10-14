use std::cell::OnceCell;

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
struct WideChar {
  utf16_start: u32,
  byte_start: u32,
}

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
enum WideChars {
  Sparse {
    chars: Vec<WideChar>,
    utf16_len: usize,
  },
  Dense(Vec<u32>),
}

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct WithUtf16<'text> {
  #[cfg_attr(allocative, allocative(skip))]
  pub line: &'text str,
  wide_chars: OnceCell<Option<WideChars>>,
  is_ascii: bool,
}

impl<'text> WithUtf16<'text> {
  pub fn new(line: &'text str) -> Self {
    Self::with_known(line, false)
  }

  pub fn with_known(line: &'text str, is_ascii: bool) -> Self {
    debug_assert!(!is_ascii || line.is_ascii());
    Self {
      line,
      wide_chars: OnceCell::new(),
      is_ascii,
    }
  }

  /// Slice by UTF-16 columns, rounding a position inside a surrogate pair down
  /// to the start of its UTF-8 character.
  #[allow(unsafe_code)]
  pub fn substring(&self, start_utf16_index: usize, end_utf16_index: usize) -> &'text str {
    if end_utf16_index <= start_utf16_index {
      return "";
    }

    let wide_chars = self.wide_chars.get_or_init(|| {
      if self.is_ascii && self.line.is_ascii() {
        return None;
      }

      let utf16_len = simd_utf16_len::utf16_len(self.line);
      if utf16_len == self.line.len() {
        return None;
      }

      // The byte/UTF-16 length gap detects dense Unicode before allocating.
      // Every wide character contributes at most two extra bytes.
      if u32::try_from(self.line.len()).is_ok()
        && self.line.len() - utf16_len >= utf16_len.div_ceil(2)
      {
        let mut indices = Vec::with_capacity(utf16_len);
        let bytes = self.line.as_bytes();
        let mut byte_start = 0;
        while byte_start < bytes.len() {
          let first_byte = bytes[byte_start];
          indices.push(byte_start as u32);
          let byte_len = first_byte.leading_ones().max(1) as usize;
          if byte_len == 4 {
            indices.push(byte_start as u32);
          }
          byte_start += byte_len;
        }
        return Some(WideChars::Dense(indices));
      }

      // All sparse offsets fit in u32, including the UTF-16 length.
      assert!(u32::try_from(self.line.len()).is_ok());

      // Like rust-analyzer's LineIndex, store only non-ASCII characters. Their
      // byte and UTF-16 offsets let ASCII spans use direct offset arithmetic.
      let mut chars = Vec::with_capacity(self.line.len() - utf16_len);
      let bytes = self.line.as_bytes();
      let mut byte_start = 0;
      let mut utf16_pos = 0;
      while byte_start < bytes.len() {
        let first_byte = bytes[byte_start];
        if first_byte < 0x80 {
          byte_start += 1;
          utf16_pos += 1;
          continue;
        }
        chars.push(WideChar {
          utf16_start: utf16_pos as u32,
          byte_start: byte_start as u32,
        });
        let byte_len = first_byte.leading_ones() as usize;
        byte_start += byte_len;
        utf16_pos += if byte_len == 4 { 2 } else { 1 };
      }
      Some(WideChars::Sparse {
        chars,
        utf16_len: utf16_pos,
      })
    });

    let Some(wide_chars) = wide_chars else {
      let start = start_utf16_index.min(self.line.len());
      let end = end_utf16_index.min(self.line.len());
      // SAFETY: No index is built only for ASCII text. Both clamped offsets
      // are therefore UTF-8 boundaries, and the early return ensures start <= end.
      return unsafe { self.line.get_unchecked(start..end) };
    };

    let to_byte = |index: usize| match wide_chars {
      WideChars::Dense(indices) => indices
        .get(index)
        .map(|&byte_start| byte_start as usize)
        .unwrap_or(self.line.len()),
      WideChars::Sparse { chars, utf16_len } => {
        if index >= *utf16_len {
          return self.line.len();
        }
        let position = chars.partition_point(|ch| ch.utf16_start as usize <= index);
        let Some(wide) = position.checked_sub(1).and_then(|i| chars.get(i)) else {
          return index;
        };
        let byte_start = wide.byte_start as usize;
        let byte_len = self.line.as_bytes()[byte_start].leading_ones() as usize;
        let utf16_len = if byte_len == 4 { 2 } else { 1 };
        let after = index - wide.utf16_start as usize;
        if after < utf16_len {
          byte_start
        } else {
          byte_start + byte_len + after - utf16_len
        }
      }
    };

    let start = to_byte(start_utf16_index);
    let end = to_byte(end_utf16_index);
    // SAFETY: Dense entries are character starts. Sparse entries are character
    // starts or offsets into ASCII runs; positions inside a surrogate pair
    // round down. The conversion is monotonic and clamps at line.len().
    unsafe { self.line.get_unchecked(start..end) }
  }
}

#[cfg(test)]
mod tests {
  use super::WithUtf16;

  #[test]
  fn test_substring() {
    assert_eq!(WithUtf16::new("foobar").substring(0, 3), "foo");
  }

  #[test]
  fn test_out_of_bounds() {
    assert_eq!(WithUtf16::new("foobar").substring(0, 10), "foobar");
    assert_eq!(WithUtf16::new("foobar").substring(6, 10), "");
  }

  #[test]
  fn test_start_less_than_end() {
    assert_eq!(WithUtf16::new("foobar").substring(3, 2), "");
  }

  #[test]
  fn test_start_and_end_equal() {
    assert_eq!(WithUtf16::new("foobar").substring(3, 3), "");
  }

  #[test]
  fn test_multiple_byte_characters() {
    assert_eq!(WithUtf16::new("🙈🙉🙊🐒").substring(2, 4), "🙉");
  }
}
