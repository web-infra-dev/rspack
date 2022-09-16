// use rspack_error::Result;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  BoxModule, Module, ModuleType, Parser, SourceType,
};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

#[derive(Debug, Default)]
pub struct AssetSourceParser {}

impl Parser for AssetSourceParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: rspack_core::ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    let boxed: BoxModule = Box::new(AssetSourceModule::new(args.source));
    Ok(boxed.with_empty_diagnostic())
  }
}
static ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];
#[derive(Debug)]
struct AssetSourceModule {
  source: BoxSource,
  source_type_list: &'static [SourceType; 1],
}

impl AssetSourceModule {
  fn new(source: BoxSource) -> Self {
    Self {
      source,
      source_type_list: ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST,
    }
  }
}

impl Module for AssetSourceModule {
  fn module_type(&self) -> ModuleType {
    ModuleType::Asset
  }

  fn source_types(&self) -> &[SourceType] {
    self.source_type_list.as_ref()
  }

  fn original_source(&self) -> &dyn Source {
    self.source.as_ref()
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<BoxSource>> {
    let result = match requested_source_type {
      SourceType::JavaScript => {
        let source = self.source.source();
        if source.is_empty() {
          None
        } else {
          Some(
            RawSource::from(format!(
              r#"function (module, exports, __rspack_require__, __rspack_dynamic_require__) {{
  "use strict";
  module.exports = {:?};
}};
"#,
              source
            ))
            .boxed(),
          )
        }
      }
      _ => None,
    };

    Ok(result)
  }
}
