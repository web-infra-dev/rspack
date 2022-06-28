// mod js_module;
// pub use js_module::*;

use crate::{module::CssModule, SWC_COMPILER};
use dashmap::DashSet;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetFilename, JobContext, Module, ParseModuleArgs, Plugin, PluginParseModuleHookOutput,
  SourceType,
};

use swc_common::{Globals, Mark, GLOBALS};

#[derive(Debug, Default)]
pub struct CssPlugin {}

static CSS_GLOBALS: Lazy<Globals> = Lazy::new(Globals::new);

// impl Default for CssPlugin {
//   fn default() -> Self {
//     Self {}
//   }
// }

impl Plugin for CssPlugin {
  fn register_parse_module(&self, _ctx: rspack_core::PluginContext) -> Option<Vec<SourceType>> {
    Some(vec![SourceType::Css])
  }

  fn parse_module(
    &self,
    ctx: rspack_core::PluginContext<&mut JobContext>,
    args: ParseModuleArgs,
  ) -> PluginParseModuleHookOutput {
    let stylesheet = SWC_COMPILER.parse_file(args.uri, args.source)?;
    // ctx.context.source_type = Some(SourceType::Js);
    Ok(Box::new(CssModule { ast: stylesheet }))
  }

  fn render_manifest(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> rspack_core::PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;
    let ordered_modules = chunk.ordered_modules(module_graph);
    let code: String = ordered_modules
      .par_iter()
      .filter(|module| matches!(module.source_type, SourceType::Css))
      .map(|module| module.module.render(module, compilation))
      .fold(String::new, |mut output, cur| {
        output += "\n\n";
        output += cur.trim();
        output
      })
      .collect();
    if code.is_empty() {
      Ok(vec![])
    } else {
      Ok(vec![Asset::new(
        code,
        AssetFilename::Static(format!("{}.css", args.chunk_id)),
      )])
    }
  }
}
