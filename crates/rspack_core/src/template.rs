use once_cell::sync::Lazy;
use regex::Regex;

pub struct Template;
static COMMENT_END_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\*\/").expect("should construct regex"));

static IDENTIFIER_NAME_REPLACE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^a-zA-Z$_])").expect("should init regex"));

static IDENTIFIER_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^a-zA-Z0-9$]+").expect("should init regex"));

impl Template {
  pub fn to_comment(str: &str) -> String {
    if str.is_empty() {
      return String::new();
    }
    format!("/*! {} */", COMMENT_END_REGEX.replace(str, "* /"))
  }

  pub fn to_normal_comment(str: &str) -> String {
    if str.is_empty() {
      return String::new();
    }
    format!("/* {} */", COMMENT_END_REGEX.replace(str, "* /"))
  }

  #[inline]
  pub fn to_identifier(v: &str) -> String {
    let id = IDENTIFIER_NAME_REPLACE_REGEX.replace_all(v, "_$1");
    IDENTIFIER_REGEXP.replace_all(&id, "_").to_string()
  }
}
