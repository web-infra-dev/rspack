use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use rspack_paths::Utf8Path;
use rspack_util::identifier::absolute_to_request;

use crate::{Context, ModuleRuleUseLoader, ResolveResult, Resolver};

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

pub async fn stringify_loaders_and_resource<'a>(
  loaders: &'a [ModuleRuleUseLoader],
  resource: &'a str,
  context: &Context,
  loader_resolve: &Resolver,
) -> Cow<'a, str> {
  if !loaders.is_empty() {
    let resolve_futures = loaders
      .iter()
      .map(|loader| {
        let loader_request = &loader.loader;
        async move {
          (
            loader_resolve
              .resolve(context.as_path().as_std_path(), loader_request)
              .await,
            loader_request,
          )
        }
      })
      .collect::<Vec<_>>();
    let resolved_loaders = futures::future::join_all(resolve_futures).await;
    let mut s = String::new();
    for (resolve_result, original_request) in resolved_loaders {
      if !s.is_empty() {
        s.push('!');
      }
      if let Ok(ResolveResult::Resource(resource)) = resolve_result {
        s.push_str(&resource.full_path());
      } else {
        s.push_str(original_request);
      }
    }
    s.push('!');
    s.push_str(resource);
    Cow::Owned(s)
  } else {
    Cow::Borrowed(resource)
  }
}
