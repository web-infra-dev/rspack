use json::Error::{
  ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
};
use rspack_core::{
  rspack_sources::{RawSource, Source, SourceExt},
  BuildMetaDefaultObject, BuildMetaExportsType, GenerateContext, Module, ParserAndGenerator,
  Plugin, SourceType,
};
use rspack_error::{
  internal_error, DiagnosticKind, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
  TraceableError,
};

mod utils;

#[derive(Debug)]
struct JsonParserAndGenerator;

impl ParserAndGenerator for JsonParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn size(&self, module: &dyn Module, _source_type: &SourceType) -> f64 {
    module.original_source().map_or(0, |source| source.size()) as f64
  }

  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> Result<TWithDiagnosticArray<rspack_core::ParseResult>> {
    let rspack_core::ParseContext {
      source: box_source,
      resource_data,
      build_info,
      build_meta,
      ..
    } = parse_context;
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;
    // TODO default_object is not align with webpack
    build_meta.default_object = BuildMetaDefaultObject::RedirectWarn;
    let source = box_source.source();

    let parse_result = json::parse(&source).map_err(|e| {
      match e {
        UnexpectedCharacter { ch, line, column } => {
          let rope = ropey::Rope::from_str(&source);
          let line_offset = rope.try_line_to_byte(line - 1).expect("TODO:");
          let start_offset = source[line_offset..]
            .chars()
            .take(column)
            .fold(line_offset, |acc, cur| acc + cur.len_utf8());
          Error::TraceableError(
            TraceableError::from_file(
              resource_data.resource_path.to_string_lossy().to_string(),
              source.into_owned(),
              // one character offset
              start_offset,
              start_offset + 1,
              "Json parsing error".to_string(),
              format!("Unexpected character {ch}"),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
        ExceededDepthLimit | WrongType(_) | FailedUtf8Parsing => {
          internal_error!(format!("{e}"))
        }
        UnexpectedEndOfJson => {
          // End offset of json file
          let offset = source.len();
          Error::TraceableError(
            TraceableError::from_file(
              resource_data.resource_path.to_string_lossy().to_string(),
              source.into_owned(),
              offset,
              offset,
              "Json parsing error".to_string(),
              format!("{e}"),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
      }
    });

    let diagnostics = if let Err(err) = parse_result {
      err.into()
    } else {
      vec![]
    };

    Ok(
      rspack_core::ParseResult {
        presentational_dependencies: vec![],
        dependencies: vec![],
        ast_or_source: box_source.into(),
      }
      .with_diagnostic(diagnostics),
    )
  }

  // Safety: `ast_and_source` is available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &rspack_core::AstOrSource,
    _module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<rspack_core::GenerationResult> {
    match generate_context.requested_source_type {
      SourceType::JavaScript => Ok(rspack_core::GenerationResult {
        ast_or_source: RawSource::from(format!(
          r#"module.exports = {};"#,
          utils::escape_json(
            &ast_or_source
              .as_source()
              .expect("Expected source for JSON generator, please file an issue.")
              .source()
          )
        ))
        .boxed()
        .into(),
      }),
      _ => Err(internal_error!(format!(
        "Unsupported source type {:?} for plugin Json",
        generate_context.requested_source_type,
      ))),
    }
  }
}

#[derive(Debug)]
pub struct JsonPlugin;

impl Plugin for JsonPlugin {
  fn name(&self) -> &'static str {
    "json"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Json,
      Box::new(|| Box::new(JsonParserAndGenerator {})),
    );

    Ok(())
  }
}
