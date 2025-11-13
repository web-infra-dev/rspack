use std::sync::Arc;

use rspack_core::{
  ChunkInitFragments, ChunkUkey, CodeGenerationDataFilename, Compilation, CompilationParams,
  CompilerCompilation, DependencyId, JavascriptParserUrl, Module, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin, URLStaticMode,
  rspack_sources::ReplaceSource,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
  dependency::{URL_STATIC_PLACEHOLDER, URL_STATIC_PLACEHOLDER_RE},
  parser_and_generator::JavaScriptParserAndGenerator,
};

#[plugin]
#[derive(Debug, Default)]
pub struct URLPlugin {}

#[plugin_hook(CompilerCompilation for URLPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .write()
    .await
    .render_module_content
    .tap(render_module_content::new(self));
  Ok(())
}
#[plugin_hook(NormalModuleFactoryParser for URLPlugin)]
async fn normal_module_factory_parser(
  &self,
  _module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>() {
    let options = parser_options
      .and_then(|p| p.get_javascript())
      .expect("should at least have a global javascript parser options");

    if !matches!(options.url, Some(JavascriptParserUrl::Disable)) {
      parser.add_parser_plugin(Box::new(crate::parser_plugin::URLPlugin {
        mode: options.url,
      }));
    }
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for URLPlugin,tracing=false)]
async fn render_module_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &dyn Module,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let runtime = compilation.chunk_by_ukey.expect_get(chunk_ukey).runtime();
  let module_graph = compilation.get_module_graph();
  let codegen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(runtime));
  if codegen_result.data.contains::<URLStaticMode>() {
    let content = render_source.source.source().into_string_lossy();
    let mut replace_source = ReplaceSource::new(render_source.source.clone());
    let replacement = URL_STATIC_PLACEHOLDER_RE
      .find_iter(&content)
      .map(|cap| (cap.start(), cap.end()));

    for (start, end) in replacement {
      let dep_id = &content[start + URL_STATIC_PLACEHOLDER.len()..end];
      let dep_id: DependencyId = dep_id
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("should be valid dependency id \"{dep_id}\""))
        .into();
      let Some(module) = module_graph.module_identifier_by_dependency_id(&dep_id) else {
        continue;
      };
      let codegen_result = compilation
        .code_generation_results
        .get(module, Some(runtime));
      let Some(filename) = codegen_result.data.get::<CodeGenerationDataFilename>() else {
        unreachable!()
      };

      replace_source.replace(start as u32, end as u32, filename.filename(), None);
    }

    render_source.source = Arc::new(replace_source);
  }
  Ok(())
}

impl Plugin for URLPlugin {
  fn name(&self) -> &'static str {
    "rspack.URLPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(normal_module_factory_parser::new(self));
    Ok(())
  }
}
