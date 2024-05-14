use std::path::PathBuf;

use rspack_core::{ChunkGraph, Dependency, Module, ModuleGraph, ParserAndGenerator};
use rspack_error::TWithDiagnosticArray;
use rustc_hash::FxHashMap;

use crate::css_dependency::CssDependency;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExtractJsonData {
  pub identifier: String,
  pub content: String,
  pub context: String,
  pub media: String,
  pub supports: String,
  pub source_map: String,
  pub identifier_index: u32,
  pub filepath: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExtractJsonDataList(pub Vec<CssExtractJsonData>);

#[derive(Debug)]
pub(crate) struct CssExtractParserAndGenerator {
  orig_parser_generator: Box<dyn ParserAndGenerator>,
  #[allow(clippy::vec_box)]
  cache: FxHashMap<CssExtractJsonDataList, Vec<Box<CssDependency>>>,
}

impl CssExtractParserAndGenerator {
  pub(crate) fn new(orig_parser_generator: Box<dyn ParserAndGenerator>) -> Self {
    Self {
      orig_parser_generator,
      cache: Default::default(),
    }
  }
}

impl ParserAndGenerator for CssExtractParserAndGenerator {
  fn source_types(&self) -> &[rspack_core::SourceType] {
    self.orig_parser_generator.source_types()
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<String> {
    None
  }

  #[allow(clippy::unwrap_used)]
  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> rspack_error::Result<rspack_error::TWithDiagnosticArray<rspack_core::ParseResult>> {
    let deps = if let Some(additional_data) = parse_context
      .additional_data
      .get::<CssExtractJsonDataList>()
    {
      if let Some(deps) = self.cache.get(additional_data) {
        deps.clone()
      } else {
        let mut idx = 0;
        let deps = additional_data
          .0
          .iter()
          .map(
            |CssExtractJsonData {
               identifier,
               content,
               context,
               media,
               supports,
               source_map,
               identifier_index,
               filepath,
             }| {
              let dep = Box::new(CssDependency::new(
                identifier.into(),
                content.clone(),
                context.clone(),
                media.clone(),
                supports.clone(),
                source_map.clone(),
                *identifier_index,
                idx,
                filepath.clone(),
              ));
              idx += 1;
              dep
            },
          )
          .collect::<Vec<_>>();
        self.cache.insert(additional_data.clone(), deps.clone());
        deps
      }
    } else {
      vec![]
    };

    let result = self.orig_parser_generator.parse(parse_context);

    if let Ok(result) = result {
      let (mut res, diags) = result.split_into_parts();

      res
        .dependencies
        .extend(deps.into_iter().map(|dep| dep as Box<dyn Dependency>));

      Ok(TWithDiagnosticArray::new(res, diags))
    } else {
      result
    }
  }

  fn size(&self, module: &dyn rspack_core::Module, source_type: &rspack_core::SourceType) -> f64 {
    self.orig_parser_generator.size(module, source_type)
  }

  fn generate(
    &self,
    source: &rspack_core::rspack_sources::BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut rspack_core::GenerateContext,
  ) -> rspack_error::Result<rspack_core::rspack_sources::BoxSource> {
    self
      .orig_parser_generator
      .generate(source, module, generate_context)
  }
}
