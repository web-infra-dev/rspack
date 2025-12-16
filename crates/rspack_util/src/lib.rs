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
  collections::hash_map::DefaultHasher,
  future::Future,
  hash::{Hash, Hasher},
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
  serde_json::to_string_pretty(v)
    .unwrap_or_else(|e| panic!("{e}: {v:?} should able to json stringify"))
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
