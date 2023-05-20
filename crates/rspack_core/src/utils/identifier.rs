use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_util::identifier::absolute_to_request;

use crate::BoxLoader;

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

static PATH_QUERY_FRAGMENT_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new("^((?:\0.|[^?#\0])*)(\\?(?:\0.|[^#\0])*)?(#.*)?$")
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
  loaders: &'a [BoxLoader],
  resource: &'a str,
) -> Cow<'a, str> {
  if !loaders.is_empty() {
    let s = loaders
      .iter()
      .map(|i| i.identifier().as_str())
      .collect::<Vec<_>>()
      .join("!");
    Cow::Owned(format!("{s}!{}", resource))
  } else {
    Cow::Borrowed(resource)
  }
}
