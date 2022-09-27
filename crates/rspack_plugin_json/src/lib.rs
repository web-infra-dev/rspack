use json::Error::{
  ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
};
use rspack_error::{
  DiagnosticKind, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray, TraceableError,
};

use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  BoxModule, Module, ModuleType, Parser, ParserAndGenerator, Plugin, SourceType,
};

mod utils;

#[derive(Debug)]
struct JsonParserAndGenerator {}

impl ParserAndGenerator for JsonParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> Result<TWithDiagnosticArray<rspack_core::ParseResult>> {
    let rspack_core::ParseContext {
      source: box_source,
      resource_data,
      ..
    } = parse_context;
    let source = box_source.source();

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
              resource_data.resource_path.to_owned(),
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
              resource_data.resource_path.to_owned(),
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

    Ok(
      rspack_core::ParseResult {
        dependencies: vec![],
        ast_or_source: box_source.into(),
      }
      .with_empty_diagnostic(),
    )
  }

  fn generate(
    &self,
    requested_source_type: SourceType,
    ast_or_source: &rspack_core::AstOrSource,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<rspack_core::GenerationResult> {
    match requested_source_type {
      SourceType::JavaScript => Ok(rspack_core::GenerationResult {
        ast_or_source: RawSource::from(format!(
          r#"module.exports = {};"#,
          utils::escape_json(&ast_or_source.as_source().unwrap().source())
        ))
        .boxed()
        .into(),
      }),
      _ => Err(Error::InternalError(format!(
        "Unsupported source type {:?} for plugin Json",
        requested_source_type,
      ))),
    }
  }
}

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
    // ctx
    //   .context
    //   .register_parser(rspack_core::ModuleType::Json, Box::new(JsonParser::new()));

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Json,
      Box::new(|| Box::new(JsonParserAndGenerator {})),
    );

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
    let source = args.source.source();

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

    let module: BoxModule = Box::new(JsonModule::new(args.source));
    Ok(module.with_empty_diagnostic())
  }
}

static JSON_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];
#[derive(Debug)]
struct JsonModule {
  module_type: ModuleType,
  source: BoxSource,
  source_type_list: &'static [SourceType; 1],
}

impl JsonModule {
  fn new(source: BoxSource) -> Self {
    Self {
      module_type: ModuleType::Json,
      source,
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

  fn original_source(&self) -> &dyn Source {
    self.source.as_ref()
  }

  #[tracing::instrument(skip_all)]
  fn render(
    &self,
    requested_source_type: SourceType,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<BoxSource>> {
    let result = match requested_source_type {
      SourceType::JavaScript => Some(
        RawSource::from(format!(
          r#"module.exports = {};"#,
          utils::escape_json(&self.source.source())
        ))
        .boxed(),
      ),
      _ => None,
    };

    Ok(result)
  }
}
