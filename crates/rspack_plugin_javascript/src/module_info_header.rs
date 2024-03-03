use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{Context, Module};
use rspack_error::Result;
use rspack_sources::{BoxSource, ConcatSource, RawSource};

static COMMENT_END_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\*/").expect("Failed to initialize COMMENT_END_REGEX"));

pub(crate) fn render_module_package(
  module_source: BoxSource,
  module: &dyn Module,
  context: &Context,
) -> Result<BoxSource> {
  let mut source = ConcatSource::default();
  let req = module.readable_identifier(context);
  let req_str = COMMENT_END_REGEX.replace_all(&req, "*_/");
  let req_str_star = "*".repeat(req_str.len());
  let header_str = format!(
    "/*!****{}****!*\\\n  !*** {} ***!\n  \\****{}****/\n",
    req_str_star, req_str, req_str_star
  );
  let header = RawSource::from(header_str);
  source.add(header);
  source.add(module_source);
  Ok(Arc::new(source))
}
