use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;
use rspack_paths::Utf8Path;
use rspack_util::identifier::absolute_to_request;

use crate::ModuleRuleUseLoader;

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
    Cow::Borrowed(_) => IDENTIFIER_REGEXP.replace_all(v, "_"),
    Cow::Owned(id) => match IDENTIFIER_REGEXP.replace_all(&id, "_") {
      Cow::Borrowed(_unchanged) => Cow::Owned(id),
      Cow::Owned(id) => Cow::Owned(id),
    },
  }
}

pub fn stringify_loaders_and_resource<'a>(
  loaders: &'a [ModuleRuleUseLoader],
  resource: &'a str,
) -> Cow<'a, str> {
  if !loaders.is_empty() {
    let s = loaders
      .iter()
      .map(|i| &*i.loader)
      .collect::<Vec<_>>()
      .join("!");
    Cow::Owned(format!("{s}!{resource}"))
  } else {
    Cow::Borrowed(resource)
  }
}
