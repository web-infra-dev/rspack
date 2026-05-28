use std::cell::OnceCell;

use crate::object_pool::{ObjectPool, Pooled};

#[derive(Debug)]
pub struct WithUtf16<'object_pool, 'text> {
  /// line is a string reference
  pub line: &'text str,
  /// the byte position of each `char` in `line` string slice .
  pub utf16_byte_indices: OnceCell<Option<Pooled<'object_pool>>>,
  is_ascii: bool,
  object_pool: &'object_pool ObjectPool,
}

impl<'object_pool, 'text> WithUtf16<'object_pool, 'text> {
  pub fn new(object_pool: &'object_pool ObjectPool, line: &'text str) -> Self {
    Self::with_known(object_pool, line, false)
  }

  pub fn with_known(
    object_pool: &'object_pool ObjectPool,
    line: &'text str,
    is_ascii: bool,
  ) -> Self {
    debug_assert!(!is_ascii || line.is_ascii());
    Self {
      utf16_byte_indices: OnceCell::new(),
      line,
      is_ascii,
      object_pool,
    }
  }

  /// substring::SubString with cache
  #[allow(unsafe_code)]
  pub fn substring(&self, start_utf16_index: usize, end_utf16_index: usize) -> &'text str {
    if end_utf16_index <= start_utf16_index {
      return "";
    }

    let utf16_byte_indices = self.utf16_byte_indices.get_or_init(|| {
      if self.is_ascii || self.line.is_ascii() {
        return None;
      }

      let mut vec = self.object_pool.pull(self.line.len());
      let bytes = self.line.as_bytes();
      let mut byte_pos = 0;
      while byte_pos < bytes.len() {
        let byte = unsafe { *bytes.get_unchecked(byte_pos) };
        if byte < 0x80 {
          // ASCII: 1 byte = 1 UTF-16 unit
          vec.push(byte_pos);
          byte_pos += 1;
        } else if byte < 0xE0 {
          // 2-byte UTF-8 = 1 UTF-16 unit
          vec.push(byte_pos);
          byte_pos += 2;
        } else if byte < 0xF0 {
          // 3-byte UTF-8 = 1 UTF-16 unit
          vec.push(byte_pos);
          byte_pos += 3;
        } else {
          // 4-byte UTF-8 = 2 UTF-16 units (surrogate pair)
          vec.push(byte_pos);
          vec.push(byte_pos);
          byte_pos += 4;
        }
      }
      Some(vec)
    });

    let utf8_len = self.line.len();

    let Some(utf16_byte_indices) = utf16_byte_indices else {
      let start_utf16_index = start_utf16_index.min(utf8_len);
      let end_utf16_index = end_utf16_index.min(utf8_len);
      return unsafe { self.line.get_unchecked(start_utf16_index..end_utf16_index) };
    };

    let start = *utf16_byte_indices
      .get(start_utf16_index)
      .unwrap_or(&utf8_len);
    let end = *utf16_byte_indices.get(end_utf16_index).unwrap_or(&utf8_len);

    unsafe {
      // SAFETY: Since `indices` iterates over the `CharIndices` of `self`, we can guarantee
      // that the indices obtained from it will always be within the bounds of `self` and they
      // will always lie on UTF-8 sequence boundaries.
      self.line.get_unchecked(start..end)
    }
  }
}

/// tests are just copy from `substring` crate
#[cfg(test)]
mod tests {
  use super::WithUtf16;
  use crate::object_pool::ObjectPool;
  #[test]
  fn test_substring() {
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "foobar").substring(0, 3),
      "foo"
    );
  }

  #[test]
  fn test_out_of_bounds() {
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "foobar").substring(0, 10),
      "foobar"
    );
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "foobar").substring(6, 10),
      ""
    );
  }

  #[test]
  fn test_start_less_than_end() {
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "foobar").substring(3, 2),
      ""
    );
  }

  #[test]
  fn test_start_and_end_equal() {
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "foobar").substring(3, 3),
      ""
    );
  }

  #[test]
  fn test_multiple_byte_characters() {
    assert_eq!(
      WithUtf16::new(&ObjectPool::default(), "🙈🙉🙊🐒").substring(2, 4),
      "🙉"
    );
  }
}
