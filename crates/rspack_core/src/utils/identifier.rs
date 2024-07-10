use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_util::identifier::absolute_to_request;

use crate::ModuleRuleUseLoader;

pub fn contextify(context: impl AsRef<Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(&context.to_string_lossy(), r))
    .collect::<Vec<Cow<str>>>()
    .join("!")
}

static IDENTIFIER_NAME_REPLACE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^a-zA-Z$_])").expect("should init regex"));
static IDENTIFIER_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^a-zA-Z0-9$]+").expect("should init regex"));

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

static PATH_QUERY_FRAGMENT_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^((?:\u{200b}.|[^?#\u{200b}])*)(\\?(?:\u{200b}.|[^#\u{200b}])*)?(#.*)?$")
    .expect("Failed to initialize `PATH_QUERY_FRAGMENT_REGEXP`")
});

#[derive(Debug)]
pub struct ResourceParsedData {
  pub path: PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
}

pub fn parse_resource(resource: &str) -> Option<ResourceParsedData> {
  let groups = PATH_QUERY_FRAGMENT_REGEXP.captures(resource)?;

  Some(ResourceParsedData {
    path: groups.get(1)?.as_str().into(),
    query: groups.get(2).map(|q| q.as_str().to_owned()),
    fragment: groups.get(3).map(|q| q.as_str().to_owned()),
  })
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
    Cow::Owned(format!("{s}!{}", resource))
  } else {
    Cow::Borrowed(resource)
  }
}
