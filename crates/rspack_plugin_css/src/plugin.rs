// mod js_module;
// pub use js_module::*;

use crate::{module::CssModule, SWC_COMPILER};
use anyhow::Context;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetFilename, Module, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, Parser,
  Plugin, PluginParseModuleHookOutput, RspackAst, TransformResult,
};
use std::path::Path;
use swc_common::{Globals, Mark, GLOBALS};

use swc_css::codegen::{
  writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  CodegenConfig, Emit,
};
use swc_css::visit::VisitMutWith;
use swc_css_prefixer::prefixer;
#[derive(Debug, Default)]
pub struct CssPlugin {}

static CSS_GLOBALS: Lazy<Globals> = Lazy::new(Globals::new);

// impl Default for CssPlugin {
//   fn default() -> Self {
//     Self {}
//   }
// }

impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }
  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> anyhow::Result<()> {
    ctx
      .context
      .register_parser(ModuleType::Css, Box::new(CssParser {}));
    Ok(())
  }
  fn reuse_ast(&self) -> bool {
    true
  }
  fn transform_include(&self, uri: &str) -> bool {
    let extension = Path::new(uri).extension().unwrap().to_string_lossy();
    extension == "css"
  }
  fn transform(
    &self,
    _ctx: rspack_core::PluginContext<&mut NormalModuleFactoryContext>,
    args: rspack_core::TransformArgs,
  ) -> rspack_core::PluginTransformOutput {
    if let Some(RspackAst::Css(mut ast)) = args.ast {
      ast.visit_mut_with(&mut prefixer());
      Ok({
        TransformResult {
          code: None,
          ast: Some(RspackAst::Css(ast)),
        }
      })
    } else {
      Ok({
        TransformResult {
          code: None,
          ast: args.ast,
        }
      })
    }
  }
  fn parse(&self, uri: &str, code: &str) -> rspack_core::PluginParseOutput {
    let stylesheet = SWC_COMPILER.parse_file(uri, code)?;
    Ok(RspackAst::Css(stylesheet))
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
    if let Some(RspackAst::Css(_ast)) = args.ast {
      Ok(Box::new(CssModule { ast: _ast }))
    } else {
      let stylesheet = SWC_COMPILER.parse_file(
        args.uri,
        &args
          .source
          .with_context(|| format!("source is empty for {}", args.uri))?,
      )?;
      Ok(Box::new(CssModule { ast: stylesheet }))
    }
  }
}
