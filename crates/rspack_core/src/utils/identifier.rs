use std::{borrow::Cow, path::Path};

use rspack_util::identifier::absolute_to_request;

pub fn contextify(context: impl AsRef<Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(&context.to_string_lossy(), r))
    .collect::<Vec<Cow<str>>>()
    .join("!")
}
