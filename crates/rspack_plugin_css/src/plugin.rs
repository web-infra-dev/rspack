// mod js_module;
// pub use js_module::*;

use crate::{module::CssModule, SWC_COMPILER};
use dashmap::DashSet;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetFilename, Module, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, Parser,
  Plugin, PluginParseModuleHookOutput,
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
  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> anyhow::Result<()> {
    ctx
      .context
      .register_parser(ModuleType::Css, Box::new(CssParser {}));
    Ok(())
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
      .filter(|module| matches!(module.module_type, ModuleType::Css))
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

#[derive(Debug)]
struct CssParser {}

impl Parser for CssParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> anyhow::Result<rspack_core::BoxModule> {
    let stylesheet = SWC_COMPILER.parse_file(args.uri, args.source)?;
    Ok(Box::new(CssModule { ast: stylesheet }))
  }
}
