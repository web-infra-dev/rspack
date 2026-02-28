use std::{borrow::Cow, collections::HashSet};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ChunkGraph, Compilation, GenerateContext, Module, ModuleGraph, NormalModule, ParseContext,
  ParseResult, ParserAndGenerator, RuntimeSpec, SourceType, rspack_sources::BoxSource,
};
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_hash::RspackHashDigest;
use rspack_plugin_asset::AssetParserAndGenerator;

#[cacheable]
#[derive(Debug)]
pub(crate) struct RslibAssetParserAndGenerator(pub AssetParserAndGenerator);

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for RslibAssetParserAndGenerator {
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType] {
    let mut source_types = HashSet::new();
    let module_id = module.identifier();
    for connection in module_graph.get_incoming_connections(&module_id) {
      if let Some(module) = connection
        .original_module_identifier
        .and_then(|id| module_graph.module_by_identifier(&id))
      {
        let module_type = module.module_type();
        source_types.insert(SourceType::from(module_type));
      }
    }

    // entry resource
    if source_types.is_empty()
      && self
        .0
        .parsed_asset_config
        .as_ref()
        .is_some_and(|config| !config.is_inline() && !config.is_source())
    {
      return &[SourceType::JavaScript, SourceType::Asset];
    }

    self.0.source_types(module, module_graph)
  }

  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    self.0.parse(parse_context).await
  }

  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64 {
    self.0.size(module, source_type)
  }

  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    self.0.generate(source, module, generate_context).await
  }

  fn get_concatenation_bailout_reason(
    &self,
    module: &dyn Module,
    mg: &ModuleGraph,
    cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    self.0.get_concatenation_bailout_reason(module, mg, cg)
  }

  async fn get_runtime_hash(
    &self,
    module: &NormalModule,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    self.0.get_runtime_hash(module, compilation, runtime).await
  }
}
