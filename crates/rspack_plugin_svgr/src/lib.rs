#![deny(clippy::all)]

mod clean;
mod mapping;
mod transform;
use async_trait::async_trait;
use clean::clean;
use core::fmt::Debug;
use rspack_core::{
  ast, PluginContext, PluginTransformAstHookOutput, PluginTransformHookOutput, TransformArgs,
};
use rspack_swc::swc_ecma_visit::VisitMutWith;
pub static PLUGIN_NAME: &str = "rspack_svgr";
use rspack_core::{LoadArgs, LoadedSource, Loader, Plugin, PluginLoadHookOutput};
use std::fs::read_to_string;
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
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  #[inline]
  async fn load(&self, _ctx: &PluginContext, args: &LoadArgs) -> PluginLoadHookOutput {
    let query_start = args.id.find(|c: char| c == '?').unwrap_or(args.id.len());
    let file_path = Path::new(&args.id[..query_start]);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");

    if ext == "svg" {
      let loader = Some(Loader::Js);
      let content =
        Some(read_to_string(file_path).unwrap_or_else(|_| panic!("file not exits {:?}", args.id)));
      Ok(Some(LoadedSource { loader, content }))
    } else {
      Ok(None)
    }
  }
  #[inline]
  fn transform_include(&self, id: &str) -> bool {
    let file_path = Path::new(&id);
    let ext = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("js");
    ext == "svg"
  }
  fn transform_ast(
    &self,
    _ctx: &PluginContext,
    _path: &str,
    mut ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    ast.visit_mut_with(&mut SvgrReplacer {});
    Ok(ast)
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
    // println!("result:\n{}", result);
    Ok(result.into())
  }
}
