use std::{borrow::Cow, sync::LazyLock};

use base64_simd::{Base64 as Raw, Error, STANDARD};
use regex::Regex;

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

// shared regex to strip non-base64 characters before decoding
static INVALID_BASE64_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^+/0-9A-Za-z-_]").expect("Invalid RegExp"));

// modified from https://github.com/feross/buffer/blob/795bbb5bda1b39f1370ebd784bea6107b087e3a7/index.js#L1942
// Buffer.from in nodejs will clean base64 first, which causes some inconsistent behavior with base64_simd
// e.g. Buffer.from("abcd?#iefix", "base64").toString("base64")
pub fn clean_base64(value: &str) -> Option<Cow<'_, str>> {
  let value = value.split('=').next()?;
  let value = value.trim();
  let value = INVALID_BASE64_RE.replace_all(value, "");
  if value.len() < 2 {
    return Some(Cow::from(""));
  }
  let value = value.into_owned();
  let len = value.len();
  let remainder = len % 4;
  if remainder == 0 {
    return Some(Cow::from(value));
  }
  let pad_len = 4 - remainder;
  if pad_len == 1 {
    let mut padded = value;
    padded.push('=');
    return Some(Cow::from(padded));
  }
  if pad_len == 2 {
    let mut padded = value;
    padded.push_str("==");
    return Some(Cow::from(padded));
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
  Some(Cow::from(value[..len - remainder].to_owned()))
}
