use std::cell::OnceCell;

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
struct WideChar {
  utf16_start: usize,
  byte_start: usize,
}

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
enum WideChars {
  One {
    ch: WideChar,
    utf16_len: usize,
  },
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
    if start_utf16_index >= self.line.len() {
      return "";
    }
    if start_utf16_index == 0 && end_utf16_index >= self.line.len() {
      return self.line;
    }

    let wide_chars = self.wide_chars.get_or_init(|| {
      if self.is_ascii {
        return None;
      }

      let utf16_len = simd_utf16_len::utf16_len(self.line);
      if utf16_len == self.line.len() {
        return None;
      }

      // The byte/UTF-16 length gap detects dense Unicode before allocating.
      // Every wide character contributes at most two extra bytes.
      if self.line.len() <= u32::MAX as usize
        && self.line.len() - utf16_len >= utf16_len.div_ceil(2)
      {
        let mut indices = Vec::with_capacity(utf16_len);
        let bytes = self.line.as_bytes();
        let mut byte_start = 0;
        while byte_start < bytes.len() {
          let first_byte = bytes[byte_start];
          indices.push(byte_start as u32);
          byte_start += if first_byte < 0x80 {
            1
          } else if first_byte < 0xe0 {
            2
          } else if first_byte < 0xf0 {
            3
          } else {
            indices.push(byte_start as u32);
            4
          };
        }
        return Some(WideChars::Dense(indices));
      }

      // Like rust-analyzer's LineIndex, store only non-ASCII characters. Their
      // byte and UTF-16 offsets let ASCII spans use direct offset arithmetic.
      let mut first = None;
      let mut chars = Vec::new();
      let mut utf16_pos = 0;
      for (byte_start, ch) in self.line.char_indices() {
        if !ch.is_ascii() {
          let wide = WideChar {
            utf16_start: utf16_pos,
            byte_start,
          };
          if let Some(first_wide) = first.take() {
            chars.push(first_wide);
            chars.push(wide);
          } else if chars.is_empty() {
            first = Some(wide);
          } else {
            chars.push(wide);
          }
        }
        utf16_pos += ch.len_utf16();
      }
      if let Some(ch) = first {
        Some(WideChars::One {
          ch,
          utf16_len: utf16_pos,
        })
      } else {
        Some(WideChars::Sparse {
          chars,
          utf16_len: utf16_pos,
        })
      }
    });

    let Some(wide_chars) = wide_chars else {
      return &self.line
        [start_utf16_index.min(self.line.len())..end_utf16_index.min(self.line.len())];
    };

    let to_byte = |index: usize| match wide_chars {
      WideChars::Dense(indices) => indices
        .get(index)
        .map(|&byte_start| byte_start as usize)
        .unwrap_or(self.line.len()),
      WideChars::One { ch, utf16_len } => {
        if index >= *utf16_len {
          return self.line.len();
        }
        if index < ch.utf16_start {
          return index;
        }
        let byte_len = self.line.as_bytes()[ch.byte_start].leading_ones() as usize;
        let utf16_len = if byte_len == 4 { 2 } else { 1 };
        let after = index - ch.utf16_start;
        if after < utf16_len {
          ch.byte_start
        } else {
          ch.byte_start + byte_len + after - utf16_len
        }
      }
      WideChars::Sparse { chars, utf16_len } => {
        if index >= *utf16_len {
          return self.line.len();
        }
        let position = chars.partition_point(|ch| ch.utf16_start <= index);
        let Some(wide) = position.checked_sub(1).and_then(|i| chars.get(i)) else {
          return index;
        };
        let byte_len = self.line.as_bytes()[wide.byte_start].leading_ones() as usize;
        let utf16_len = if byte_len == 4 { 2 } else { 1 };
        let after = index - wide.utf16_start;
        if after < utf16_len {
          wide.byte_start
        } else {
          wide.byte_start + byte_len + after - utf16_len
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
