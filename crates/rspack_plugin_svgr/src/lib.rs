#![deny(clippy::all)]

mod clean;
mod mapping;
mod transform;
use async_trait::async_trait;
use clean::clean;
use core::fmt::Debug;
use rspack_core::{
  parse_file, PluginContext, PluginTransformHookOutput, RspackAst, TransformArgs, TransformResult,
};
use rspack_swc::swc_ecma_visit::VisitMutWith;
pub static PLUGIN_NAME: &str = "rspack_svgr";
use rspack_core::{Loader, Plugin};

use std::path::Path;

use transform::SvgrReplacer;

#[derive(Debug)]
pub struct SvgrPlugin {}
impl SvgrPlugin {}
#[async_trait]
impl Plugin for SvgrPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  #[inline]
  fn need_build_start(&self) -> bool {
    false
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }
  #[inline]
  fn reuse_ast(&self) -> bool {
    false
  }
  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  #[inline]
  fn transform_include(&self, id: &str, _: &Option<Loader>) -> bool {
    let file_path = Path::new(&id);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");
    ext == "svg"
  }
  fn transform(&self, _ctx: &PluginContext, args: TransformArgs) -> PluginTransformHookOutput {
    if !_ctx.options.svgr {
      return Ok(args.into());
    }

    *args.loader = Some(Loader::Jsx);
    let result = clean(&args.code);
    let result = format!(
      r#"import * as React from "react";
const SvgComponent = (props) => {{
  return {};
}};
export default SvgComponent;"#,
      result.trim()
    )
    .trim()
    .to_string();
    let mut ast = parse_file(&result, &args.uri, &Loader::Jsx).expect_module();
    ast.visit_mut_with(&mut SvgrReplacer {});
    // println!("result:\n{}", result);
    Ok(TransformResult {
      code: result,
      ast: Some(RspackAst::JavaScript(ast)),
    })
  }
}
