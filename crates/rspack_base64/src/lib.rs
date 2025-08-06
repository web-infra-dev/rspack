pub mod base64 {
  use std::borrow::Cow;

  use base64_simd::{Base64 as Raw, Error, STANDARD};

  pub struct Base64(Raw);

  impl Base64 {
    pub const fn new() -> Self {
      Self(STANDARD)
    }

    pub fn encode_to_string<D: AsRef<[u8]>>(&self, data: D) -> String {
      self.0.encode_to_string(data)
    }

    pub fn decode_to_vec<D: AsRef<[u8]>>(&self, data: D) -> Result<Vec<u8>, Error> {
      self.0.decode_to_vec(data)
    }
  }

  impl Default for Base64 {
    fn default() -> Self {
      Self::new()
    }
  }

  static BASE64: Base64 = Base64::new();

  pub fn encode_to_string<D: AsRef<[u8]>>(data: D) -> String {
    BASE64.0.encode_to_string(data)
  }

  pub fn decode_to_vec<D: AsRef<[u8]>>(data: D) -> Result<Vec<u8>, Error> {
    BASE64.0.decode_to_vec(data)
  }


  // modified from https://github.com/feross/buffer/blob/795bbb5bda1b39f1370ebd784bea6107b087e3a7/index.js#L1942
  // Buffer.from in nodejs will clean base64 first, which causes some inconsistent behavior with base64_simd
  // e.g. Buffer.from("abcd?#iefix", "base64").toString("base64")
  pub fn clean_base64(value: &str) -> Option<Cow<'_, str>> {
    let value = value.split('=').next()?;
    let value = value.trim();
    
    // Filter out invalid base64 characters manually - this is more efficient than regex replacement
    let mut has_invalid = false;
    for byte in value.bytes() {
      if !matches!(byte, b'+'| b'/' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_') {
        has_invalid = true;
        break;
      }
    }
    
    let value = if has_invalid {
      Cow::Owned(value.chars().filter(|&c| {
        matches!(c, '+' | '/' | '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_')
      }).collect())
    } else {
      Cow::Borrowed(value)
    };
    if value.len() < 2 {
      return Some(Cow::from(""));
    }
    let len = value.len();
    let remainder = len % 4;
    if remainder == 0 {
      return Some(value);
    }
    let pad_len = 4 - remainder;
    let owned_value = value.into_owned();
    if pad_len == 1 {
      return Some(Cow::from(owned_value + "="));
    }
    if pad_len == 2 {
      return Some(Cow::from(owned_value + "=="));
    }
    // modify: add this case on the original base64clean js function
    // why Buffer.from("abcd?#iefix", "base64") => "abcdiefi"?
    //   1. base64clean("abcd?#iefix") => "abcdiefix==="
    //   2. toByteArray("abcdiefix===") => "abcdiefi"
    // but base64_simd::STANDARD.decode_to_vec("abcdiefix===") will return error
    // because toByteArray and base64_simd::STANDARD.decode_to_vec are different with handling placeHoldersLen
    // for detail checkout:
    //   - https://github.com/beatgammit/base64-js/blob/88957c9943c7e2a0f03cdf73e71d579e433627d3/index.js#L80-L96
    //   - https://docs.rs/base64-simd/0.8.0/src/base64_simd/decode.rs.html#70 (means placeHoldersLen === 3)
    Some(Cow::from(owned_value[..len - remainder].to_owned()))
  }
}

pub use base64::{clean_base64, decode_to_vec, encode_to_string};

#[cfg(test)]
mod tests {
  use super::base64::clean_base64;

  #[test]
  fn test_clean_base64_removes_invalid_chars() {
    // Test case from the comment - known behavior
    let result = clean_base64("abcd?#iefix").unwrap();
    assert_eq!(result, "abcdiefi");

    // Test simple valid case
    let result = clean_base64("abcd").unwrap();
    assert_eq!(result, "abcd");
    
    // Test case needing padding
    let result = clean_base64("abc").unwrap();
    assert_eq!(result, "abc=");
    
    // Test case needing double padding
    let result = clean_base64("ab").unwrap();
    assert_eq!(result, "ab==");
  }
}
