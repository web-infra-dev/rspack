// mod js_module;
// pub use js_module::*;

use rspack_error::Result;
use std::fmt::Debug;

use rspack_core::{
  rspack_sources::{
    BoxSource, MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
    SourceMapSourceOptions,
  },
  Module, ModuleType, SourceType,
};

use swc_css::{ast::Stylesheet, visit::VisitMutWith};

use crate::{visitors::DependencyScanner, SWC_COMPILER};

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub struct CssModule {
  pub loaded_source: BoxSource,
  pub ast: Stylesheet,
  pub source_type_list: &'static [SourceType; 2],
  pub meta: Option<String>,
}

impl Debug for CssModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CssModule").field("ast", &"...").finish()
  }
}

impl Module for CssModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    ModuleType::Css
  }

  #[inline(always)]
  fn source_types(&self) -> &[SourceType] {
    self.source_type_list.as_ref()
  }

  fn original_source(&self) -> &dyn Source {
    self.loaded_source.as_ref()
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<BoxSource>> {
    let result = match requested_source_type {
      SourceType::Css => {
        let (code, source_map) = SWC_COMPILER.codegen(
          &self.ast,
          compilation.options.devtool.then(|| self.original_source()),
        )?;
        if let Some(source_map) = source_map {
          let source = SourceMapSource::new(SourceMapSourceOptions {
            value: code,
            name: module.uri.to_string(),
            source_map: SourceMap::from_slice(&source_map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            original_source: Some(self.original_source().source().to_string()),
            inner_source_map: self.original_source().map(&MapOptions::default()),
            remove_original_source: false,
          })
          .boxed();
          Some(source)
        } else {
          Some(RawSource::from(code).boxed())
        }
      }
      // This is just a temporary solution for css-modules
      SourceType::JavaScript => Some(
        RawSource::from(
          self
            .meta
            .clone()
            .map(|item| format!("module.exports = {};", item))
            .unwrap_or_else(|| "".to_string()),
        )
        .boxed(),
      ),
      _ => None,
    };
    Ok(result)
  }

  fn dependencies(&mut self) -> Vec<rspack_core::ModuleDependency> {
    let mut scanner = DependencyScanner::default();
    self.ast.visit_mut_with(&mut scanner);
    scanner.dependencies
  }
}
