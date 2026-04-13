mod merge;

pub mod asset_condition;
pub mod atom;
pub mod base64;
pub mod comparators;
#[cfg(feature = "debug_tool")]
pub mod debug_tool;
pub mod env;
pub mod ext;
pub mod fx_hash;
pub mod identifier;
pub mod itoa;
pub mod location;
pub mod node_path;
pub mod number_hash;
pub mod queue;
pub mod ryu_js;
pub mod size;
pub mod source_map;
pub mod span;
pub mod swc;
pub mod test;
pub mod tracing_preset;

use std::{
  future::Future,
  sync::LazyLock,
  time::{SystemTime, UNIX_EPOCH},
};

#[cfg(allocative)]
pub use allocative;
pub use merge::{MergeFrom, merge_from_optional_with};
use regex::Regex;
pub use span::SpanExt;

pub async fn try_any<T, Fut, F, E>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool, E>
where
  Fut: Future<Output = Result<bool, E>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if f(i).await? {
      return Ok(true);
    }
  }
  Ok(false)
}

pub async fn try_all<T, Fut, F, E>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool, E>
where
  Fut: Future<Output = Result<bool, E>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if !(f(i).await?) {
      return Ok(false);
    }
  }
  Ok(true)
}

pub fn json_stringify<T: ?Sized + serde::Serialize + std::fmt::Debug>(v: &T) -> String {
  serde_json::to_string(v).unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
}

pub fn json_stringify_pretty<T: ?Sized + serde::Serialize + std::fmt::Debug>(v: &T) -> String {
  serde_json::to_string_pretty(v)
    .unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
}

/// JSON-stringify a string value using SIMD-accelerated escaping.
///
/// This is a faster alternative to `serde_json::to_string(s)` for `&str` inputs.
/// The output includes surrounding double quotes, e.g. `json_stringify_str("hello")` returns `"\"hello\""`.
#[inline]
pub fn json_stringify_str(s: &str) -> String {
  json_escape_simd::escape(s)
}

/// Stringify a chunk ID for JavaScript output.
///
/// If the string is a valid non-negative integer within `Number.MAX_SAFE_INTEGER`
/// (no leading zeros except "0"), it is rendered as a number literal (e.g. `903`).
/// Otherwise, it is rendered as a JSON string (e.g. `"main"`). This matches
/// webpack's behavior where numeric chunk IDs are emitted without quotes.
#[inline]
pub fn json_stringify_chunk_id(s: &str) -> String {
  if is_numeric_id(s) {
    s.to_string()
  } else {
    json_stringify_str(s)
  }
}

/// Stringify an array of chunk IDs for JavaScript output, e.g. `[903, "main"]`.
pub fn json_stringify_chunk_ids<S: AsRef<str>>(ids: &[S]) -> String {
  let mut result = String::from("[");
  for (i, id) in ids.iter().enumerate() {
    if i > 0 {
      result.push(',');
    }
    result.push_str(&json_stringify_chunk_id(id.as_ref()));
  }
  result.push(']');
  result
}

/// JavaScript's `Number.MAX_SAFE_INTEGER` (2^53 - 1).
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Check if a string represents a valid non-negative integer suitable for
/// rendering as a JS number literal (no leading zeros except "0" itself,
/// and within `Number.MAX_SAFE_INTEGER`).
fn is_numeric_id(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }
  let bytes = s.as_bytes();
  // Reject leading zeros (e.g. "01") but allow "0"
  if bytes.len() > 1 && bytes[0] == b'0' {
    return false;
  }
  if !bytes.iter().all(|b| b.is_ascii_digit()) {
    return false;
  }
  // Guard against values that exceed Number.MAX_SAFE_INTEGER
  s.parse::<u64>()
    .map(|n| n <= MAX_SAFE_INTEGER)
    .unwrap_or(false)
}

/// Get current time in milliseconds since Unix epoch
pub fn current_time() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("should get current time")
    .as_millis() as u64
}

static QUOTE_META_REG: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Failed to initialize QUOTE_META_REG")
});

/// Escape special regex characters in a string
pub fn quote_meta(str: &str) -> String {
  QUOTE_META_REG.replace_all(str, "\\$0").to_string()
}

#[cfg(test)]
mod tests {
  use super::{is_numeric_id, json_stringify_chunk_id, json_stringify_chunk_ids};

  #[test]
  fn numeric_id_detection_covers_edge_cases() {
    assert!(!is_numeric_id(""));
    assert!(is_numeric_id("0"));
    assert!(is_numeric_id("903"));
    assert!(!is_numeric_id("01"));
    assert!(!is_numeric_id("1a"));
    assert!(!is_numeric_id("main"));
    // Within MAX_SAFE_INTEGER
    assert!(is_numeric_id("9007199254740991"));
    // Exceeds MAX_SAFE_INTEGER
    assert!(!is_numeric_id("9007199254740992"));
    // Way too large
    assert!(!is_numeric_id("123456789012345678901234567890"));
  }

  #[test]
  fn json_stringify_chunk_id_preserves_numeric_and_string_forms() {
    assert_eq!(json_stringify_chunk_id(""), "\"\"");
    assert_eq!(json_stringify_chunk_id("0"), "0");
    assert_eq!(json_stringify_chunk_id("903"), "903");
    assert_eq!(json_stringify_chunk_id("01"), "\"01\"");
    assert_eq!(json_stringify_chunk_id("1a"), "\"1a\"");
    assert_eq!(json_stringify_chunk_id("main"), "\"main\"");
    // Exceeds MAX_SAFE_INTEGER → falls back to string
    assert_eq!(
      json_stringify_chunk_id("9007199254740992"),
      "\"9007199254740992\""
    );
  }

  #[test]
  fn json_stringify_chunk_ids_renders_array() {
    assert_eq!(json_stringify_chunk_ids::<&str>(&[]), "[]");
    assert_eq!(json_stringify_chunk_ids(&["903"]), "[903]");
    assert_eq!(
      json_stringify_chunk_ids(&["903", "main", "17"]),
      "[903,\"main\",17]"
    );
  }
}
