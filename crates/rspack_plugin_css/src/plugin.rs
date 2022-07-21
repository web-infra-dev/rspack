// mod js_module;
// pub use js_module::*;

use crate::{module::CssModule, SWC_COMPILER};

use anyhow::Context;
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetContent, Content, Filename, ModuleAst, ModuleRenderResult, ModuleType,
  NormalModuleFactoryContext, OutputFilename, ParseModuleArgs, Parser, Plugin, SourceType,
  TransformAst, TransformResult,
};
use std::path::Path;

use swc_css::visit::VisitMutWith;
use swc_css_prefixer::prefixer;
#[derive(Debug, Default)]
pub struct CssPlugin {}

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
    if let Some(TransformAst::Css(mut ast)) = args.ast {
      ast.visit_mut_with(&mut prefixer());
      Ok({
        TransformResult {
          content: None,
          ast: Some(TransformAst::Css(ast)),
        }
      })
    } else {
      Ok({
        TransformResult {
          content: None,
          ast: args.ast,
        }
      })
    }
  }
  fn parse(&self, uri: &str, content: &Content) -> rspack_core::PluginParseOutput {
    let content = content
      .to_owned()
      .try_into_string()
      .context("Unable to serialize content as string which is required by plugin css")?;
    let stylesheet = SWC_COMPILER.parse_file(uri, content)?;
    Ok(TransformAst::Css(stylesheet))
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
      .filter(|module| {
        module
          .module
          .source_types(module, compilation)
          .contains(&SourceType::Css)
      })
      .map(|module| module.module.render(SourceType::Css, module, compilation))
      .fold(String::new, |mut output, cur| {
        if let Ok(Some(ModuleRenderResult::Css(source))) = cur {
          output += "\n\n";
          output += &source;
        }
        output
      })
      .collect();
    if code.is_empty() {
      Ok(Default::default())
    } else {
      Ok(vec![Asset::new(
        AssetContent::String(code),
        OutputFilename::new("[name][ext]".to_owned())
          .filename(args.chunk_id.to_owned(), ".css".to_owned()),
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
    if let Some(ModuleAst::Css(_ast)) = args.ast {
      Ok(Box::new(CssModule { ast: _ast }))
    } else if let Some(content) = args.source {
      let content = content
        .try_into_string()
        .context("Unable to serialize content as string which is required by plugin css")?;
      let stylesheet = SWC_COMPILER.parse_file(args.uri, content)?;
      Ok(Box::new(CssModule { ast: stylesheet }))
    } else {
      Err(anyhow::format_err!(
        "source is empty or unmatched content type returned for {}, content type {:?}",
        args.uri,
        args.source
      ))
    }
  }
}
