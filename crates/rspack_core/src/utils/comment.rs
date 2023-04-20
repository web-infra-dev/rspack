use once_cell::sync::Lazy;
use regex::Regex;

static COMMENT_END_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\\*\\/").expect("should init regex"));

#[inline]
pub fn to_comment(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }

  let result = COMMENT_END_REGEX.replace_all(str, "* /");

  format!("/*! {} */", result)
}
