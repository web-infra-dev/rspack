use anyhow::Result;
use hashbrown::HashSet;

use rspack_core::{
  BoxModule, Content, Module, ModuleRenderResult, ModuleType, Parser, Plugin, SourceType,
};

#[derive(Debug)]
pub struct JsonPlugin {}

impl Plugin for JsonPlugin {
  fn name(&self) -> &'static str {
    "json"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    ctx
      .context
      .register_parser(rspack_core::ModuleType::Json, Box::new(JsonParser::new()));

    Ok(())
  }
}

#[derive(Debug)]
struct JsonParser {}

impl JsonParser {
  fn new() -> Self {
    Self {}
  }
}

impl Parser for JsonParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: rspack_core::ParseModuleArgs,
  ) -> Result<BoxModule> {
    let source = args.source;
    Ok(Box::new(JsonModule::new(source)))
  }
}

#[derive(Debug)]
struct JsonModule {
  module_type: ModuleType,
  source: Option<Content>,
}

impl JsonModule {
  fn new(source: Option<Content>) -> Self {
    Self {
      module_type: ModuleType::Json,
      source,
    }
  }
}

impl Module for JsonModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  fn source_types(
    &self,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> HashSet<SourceType> {
    HashSet::from_iter([SourceType::JavaScript])
  }

  #[tracing::instrument(skip_all)]
  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::JavaScript => {
        let json_str = self
          .source
          .as_ref()
          .map(|content| content.as_string())
          .transpose()?
          .unwrap_or_else(|| "{}".to_owned());

        Some(ModuleRenderResult::JavaScript(format!(
          r#"rs.define("{}", function(__rspack_require__, module, exports) {{
    "use strict";
    module.exports = {};
  }});
  "#,
          module.id,
          json::stringify(json::parse(&json_str)?)
        )))
      }
      _ => None,
    };

    Ok(result)
  }
}
