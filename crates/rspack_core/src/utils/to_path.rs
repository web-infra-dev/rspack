use once_cell::sync::Lazy;
use regex::Regex;

static PATH_NAME_NORMALIZE_REPLACE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^a-zA-Z0-9_!§$()\-=^°]+").expect("regexp failed"));

static MATCH_PADDED_HYPHENS_REPLACE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^-|-$").expect("regexp failed"));

pub fn to_path(str: &str) -> String {
  let temp = PATH_NAME_NORMALIZE_REPLACE_REGEX.replace_all(str, "-");
  let res = MATCH_PADDED_HYPHENS_REPLACE_REGEX.replace_all(temp.as_ref(), "");
  res.to_string()
}
