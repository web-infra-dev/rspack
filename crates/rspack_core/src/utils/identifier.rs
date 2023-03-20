use std::{borrow::Cow, path::Path};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_util::identifier::absolute_to_request;

pub fn contextify(context: impl AsRef<Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(&context.to_string_lossy(), r))
    .collect::<Vec<Cow<str>>>()
    .join("!")
}

static IDENTIFIER_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^a-zA-Z0-9$]+").expect("should init regex"));

#[inline]
pub fn to_identifier(v: &str) -> Cow<'_, str> {
  IDENTIFIER_REGEXP.replace_all(v, "_")
}
