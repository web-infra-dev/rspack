use anyhow::Result;
use hashbrown::HashSet;

use rspack_core::{BoxModule, Module, ModuleRenderResult, ModuleType, Parser, Plugin, SourceType};

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
    let json_str = args
      .source
      .as_ref()
      .map(|content| content.as_string())
      .transpose()?
      .map(|s| json::parse(&s).map(|_| s))
      .transpose()?
      .unwrap_or_else(|| "{}".to_owned());

    Ok(Box::new(JsonModule::new(json_str)))
  }
}

#[derive(Debug)]
struct JsonModule {
  module_type: ModuleType,
  json_str: String,
}

impl JsonModule {
  fn new(json_str: String) -> Self {
    Self {
      module_type: ModuleType::Json,
      json_str,
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
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"rs.define("{}", function(__rspack_require__, module, exports) {{
    "use strict";
    module.exports = {};
  }});
  "#,
        module.id,
        self
          .json_str
          .replace('\u{2028}', r#"\\u2028"#)
          .replace('\u{2029}', r#"\\u2029"#)
      ))),
      _ => None,
    };

    Ok(result)
  }
}
