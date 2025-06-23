use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use rspack_paths::Utf8Path;
use rspack_util::identifier::absolute_to_request;

use crate::BoxLoader;

pub fn contextify(context: impl AsRef<Utf8Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(context.as_str(), r))
    .collect::<Vec<Cow<str>>>()
    .join("!")
}

static IDENTIFIER_NAME_REPLACE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^([^a-zA-Z$_])").expect("should init regex"));
static IDENTIFIER_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9$]+").expect("should init regex"));

#[inline]
pub fn to_identifier(v: &str) -> Cow<str> {
  // Avoid any unnecessary cost
  match IDENTIFIER_NAME_REPLACE_REGEX.replace_all(v, "_$1") {
    Cow::Borrowed(_) => escape_identifier(v),
    Cow::Owned(id) => match escape_identifier(&id) {
      Cow::Borrowed(_unchanged) => Cow::Owned(id),
      Cow::Owned(id) => Cow::Owned(id),
    },
  }
}

pub fn to_identifier_with_escaped(v: String) -> String {
  if v.is_empty() {
    return v;
  }

  if let Some(first_char) = v.chars().next() {
    if first_char.is_ascii_alphabetic() || first_char == '$' || first_char == '_' {
      return v;
    }
    format!("_{v}")
  } else {
    v
  }
}

pub fn escape_identifier(v: &str) -> Cow<str> {
  IDENTIFIER_REGEXP.replace_all(v, "_")
}

pub fn stringify_loaders_and_resource<'a>(
  loaders: &'a [BoxLoader],
  resource: &'a str,
) -> Cow<'a, str> {
  if !loaders.is_empty() {
    let mut s = String::new();
    for loader in loaders {
      let identifier = loader.identifier();
      if let Some((_type, ident)) = identifier.split_once('|') {
        s.push_str(ident);
      } else {
        s.push_str(identifier.as_str());
      }
      s.push('!');
    }
    s.push_str(resource);
    Cow::Owned(s)
  } else {
    Cow::Borrowed(resource)
  }
}
