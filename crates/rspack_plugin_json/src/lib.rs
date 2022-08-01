use anyhow::Result;

use rspack_core::{BoxModule, Module, ModuleRenderResult, ModuleType, Parser, Plugin, SourceType};

mod utils;

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
      .map(|content| content.try_into_string())
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
  source_type_vec: &'static [SourceType; 1],
}

impl JsonModule {
  fn new(json_str: String) -> Self {
    Self {
      module_type: ModuleType::Json,
      json_str,
      source_type_vec: &[SourceType::JavaScript],
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
  ) -> &[SourceType] {
    self.source_type_vec.as_ref()
  }

  #[tracing::instrument(skip_all)]
  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let namespace = &compilation.options.output.namespace;
    let result = match requested_source_type {
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"self["{}"].__rspack_register__(["{}"], {{"{}": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {{
    "use strict";
    module.exports = {};
  }}}});
  "#,
        namespace,
        module.id,
        module.id,
        utils::escape_json(&self.json_str)
      ))),
      _ => None,
    };

    Ok(result)
  }
}
