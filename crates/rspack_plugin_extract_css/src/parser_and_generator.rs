use rspack_core::{Dependency, ParserAndGenerator};
use rspack_error::TWithDiagnosticArray;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::css_dependency::CssDependency;

#[derive(Deserialize)]
struct CssExtractJsonData {
  #[serde(rename = "rspack-mini-css-extract-plugin")]
  value: String,
}

#[derive(Debug)]
pub(crate) struct CssExtractParserAndGenerator {
  orig_parser_generator: Box<dyn ParserAndGenerator>,
  #[allow(clippy::vec_box)]
  cache: FxHashMap<String, Vec<Box<CssDependency>>>,
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

  #[allow(clippy::unwrap_used)]
  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> rspack_error::Result<rspack_error::TWithDiagnosticArray<rspack_core::ParseResult>> {
    let deps = if let Some(additional_data) = parse_context.additional_data.get::<String>() {
      if let Some(deps) = self.cache.get(additional_data) {
        deps.clone()
      } else if let Ok(data) = serde_json::from_str::<CssExtractJsonData>(additional_data) {
        // parse the css data from js loader
        // data:
        // [identifier]__RSPACK_CSS_EXTRACT_SEP__
        // [content]__RSPACK_CSS_EXTRACT_SEP__
        // [context]__RSPACK_CSS_EXTRACT_SEP__
        // [media]__RSPACK_CSS_EXTRACT_SEP__
        // [supports]__RSPACK_CSS_EXTRACT_SEP__
        // [sourceMap]__RSPACK_CSS_EXTRACT_SEP__
        // [identifier]__RSPACK_CSS_EXTRACT_SEP__ ... repeated
        // [content]__RSPACK_CSS_EXTRACT_SEP__
        let mut list = data.value.split("__RSPACK_CSS_EXTRACT_SEP__");

        let mut deps = vec![];
        let mut idx = 0;
        while let Some(identifier) = list.next() {
          #[allow(clippy::unwrap_in_result)]
          {
            deps.push(Box::new(CssDependency::new(
              identifier.into(),
              list.next().unwrap().into(),
              list.next().unwrap().into(),
              list.next().unwrap().into(),
              list.next().unwrap().into(),
              list.next().unwrap().into(),
              list
                .next()
                .unwrap()
                .parse()
                .expect("Cannot parse identifier_index, this should never happen"),
              idx,
              list.next().unwrap().into(),
            )));
          }
          idx += 1;
        }

        self.cache.insert(data.value.clone(), deps.clone());

        deps
      } else {
        vec![]
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
