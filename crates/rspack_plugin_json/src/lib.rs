use json::Error::{
  ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
};
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  BuildMetaDefaultObject, BuildMetaExportsType, CompilerOptions, GenerateContext, Module,
  ParserAndGenerator, Plugin, RuntimeGlobals, SourceType,
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
    let strip_bom_source = source.strip_prefix('\u{feff}');
    let need_strip_bom = strip_bom_source.is_some();

    let parse_result = json::parse(strip_bom_source.unwrap_or(&source)).map_err(|e| {
      match e {
        UnexpectedCharacter { ch, line, column } => {
          let rope = ropey::Rope::from_str(&source);
          let line_offset = rope.try_line_to_byte(line - 1).expect("TODO:");
          let start_offset = source[line_offset..]
            .chars()
            .take(column)
            .fold(line_offset, |acc, cur| acc + cur.len_utf8());
          let start_offset = if need_strip_bom {
            start_offset + 1
          } else {
            start_offset
          };
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
        source: box_source,
        analyze_result: Default::default(),
      }
      .with_diagnostic(diagnostics),
    )
  }

  // Safety: `ast_and_source` is available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    source: &BoxSource,
    _module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE);
        Ok(
          RawSource::from(format!(
            r#"module.exports = {};"#,
            utils::escape_json(&source.source())
          ))
          .boxed(),
        )
      }
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
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Json,
      Box::new(|| Box::new(JsonParserAndGenerator {})),
    );

    Ok(())
  }
}
