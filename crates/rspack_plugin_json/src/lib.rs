use json::Error::{
  ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
};
use rspack_error::{
  DiagnosticKind, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray, TraceableError,
};

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
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    let source = args.source.try_into_string()?;

    json::parse(&source).map_err(|e| {
      match e {
        UnexpectedCharacter { ch, line, column } => {
          let rope = ropey::Rope::from_str(&source);
          let line_offset = rope.try_line_to_byte(line - 1).unwrap();
          let start_offset = source[line_offset..]
            .chars()
            .take(column)
            .fold(line_offset, |acc, cur| acc + cur.len_utf8());
          Error::TraceableError(
            TraceableError::from_path(
              args.uri.to_owned(),
              // one character offset
              start_offset,
              start_offset + 1,
              "Json parsing error".to_string(),
              format!("Unexpected character {}", ch),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
        ExceededDepthLimit | WrongType(_) | FailedUtf8Parsing => {
          Error::InternalError(format!("{}", e))
        }
        UnexpectedEndOfJson => {
          // End offset of json file
          let offset = source.len();
          Error::TraceableError(
            TraceableError::from_path(
              args.uri.to_owned(),
              offset,
              offset,
              "Json parsing error".to_string(),
              format!("{}", e),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
      }
    })?;

    let module: BoxModule = Box::new(JsonModule::new(source));
    Ok(module.with_empty_diagnostic())
  }
}

static JSON_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];
#[derive(Debug)]
struct JsonModule {
  module_type: ModuleType,
  json_str: String,
  source_type_list: &'static [SourceType; 1],
}

impl JsonModule {
  fn new(json_str: String) -> Self {
    Self {
      module_type: ModuleType::Json,
      json_str,
      source_type_list: JSON_MODULE_SOURCE_TYPE_LIST,
    }
  }
}

impl Module for JsonModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  fn source_types(&self) -> &[SourceType] {
    self.source_type_list.as_ref()
  }

  #[tracing::instrument(skip_all)]
  fn render(
    &self,
    requested_source_type: SourceType,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"function (module, exports, __rspack_require__, __rspack_dynamic_require__) {{
    "use strict";
    module.exports = {};
  }};
  "#,
        utils::escape_json(&self.json_str)
      ))),
      _ => None,
    };

    Ok(result)
  }
}
