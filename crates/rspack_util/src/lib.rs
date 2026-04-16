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
/// Chunk IDs in JavaScript runtime code should stay as JSON strings, e.g. `"903"`.
#[inline]
pub fn json_stringify_chunk_id(s: &str) -> String {
  json_stringify_str(s)
}

/// Stringify an array of chunk IDs for JavaScript output, e.g. `["903", "main"]`.
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

/// Parse a string as a valid non-negative chunk/module ID suitable for rendering
/// as a JS number literal (no leading zeros except "0" itself, and within
/// `u32::MAX`).
pub fn numeric_id_value(s: &str) -> Option<u32> {
  if s.is_empty() {
    return None;
  }
  let bytes = s.as_bytes();
  // Reject leading zeros (e.g. "01") but allow "0"
  if bytes.len() > 1 && bytes[0] == b'0' {
    return None;
  }
  if !bytes.iter().all(|b| b.is_ascii_digit()) {
    return None;
  }
  s.parse::<u32>().ok()
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
  use super::{json_stringify_chunk_id, json_stringify_chunk_ids, numeric_id_value};

  #[test]
  fn numeric_id_detection_covers_edge_cases() {
    assert_eq!(numeric_id_value(""), None);
    assert_eq!(numeric_id_value("0"), Some(0));
    assert_eq!(numeric_id_value("903"), Some(903));
    assert_eq!(numeric_id_value("01"), None);
    assert_eq!(numeric_id_value("1a"), None);
    assert_eq!(numeric_id_value("main"), None);
    assert_eq!(numeric_id_value("4294967295"), Some(u32::MAX));
    // Exceeds u32::MAX
    assert_eq!(numeric_id_value("4294967296"), None);
    // Way too large
    assert_eq!(numeric_id_value("123456789012345678901234567890"), None);
  }

  #[test]
  fn json_stringify_chunk_id_preserves_numeric_and_string_forms() {
    assert_eq!(json_stringify_chunk_id(""), "\"\"");
    assert_eq!(json_stringify_chunk_id("0"), "\"0\"");
    assert_eq!(json_stringify_chunk_id("903"), "\"903\"");
    assert_eq!(json_stringify_chunk_id("01"), "\"01\"");
    assert_eq!(json_stringify_chunk_id("1a"), "\"1a\"");
    assert_eq!(json_stringify_chunk_id("main"), "\"main\"");
    assert_eq!(json_stringify_chunk_id("4294967296"), "\"4294967296\"");
  }

  #[test]
  fn json_stringify_chunk_ids_renders_array() {
    assert_eq!(json_stringify_chunk_ids::<&str>(&[]), "[]");
    assert_eq!(json_stringify_chunk_ids(&["903"]), "[\"903\"]");
    assert_eq!(
      json_stringify_chunk_ids(&["903", "main", "17"]),
      "[\"903\",\"main\",\"17\"]"
    );
  }
}
