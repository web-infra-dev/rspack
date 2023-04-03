#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(box_syntax)]

pub mod dependency;
pub mod plugin;
pub mod pxtorem;
mod utils;
pub mod visitors;

use std::sync::Arc;

use once_cell::sync::Lazy;
pub use plugin::CssPlugin;
use rspack_core::rspack_sources::{self, SourceExt};
use rspack_core::ErrorSpan;
use rspack_error::{
  internal_error, Diagnostic, DiagnosticKind, IntoTWithDiagnosticArray, Result,
  TWithDiagnosticArray,
};
use swc_core::common::{
  input::SourceFileInput, source_map::SourceMapGenConfig, FileName, SourceMap,
};
use swc_core::common::{Globals, SourceFile, GLOBALS};
use swc_core::css::minifier;
use swc_core::css::parser::{lexer::Lexer, parser::ParserConfig};
use swc_core::css::{ast::Stylesheet, parser::parser::Parser};
use swc_core::css::{
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::error::Error,
};

static SWC_COMPILER: Lazy<Arc<SwcCssCompiler>> = Lazy::new(|| Arc::new(SwcCssCompiler::new()));

#[derive(Default)]
pub struct SwcCssCompiler {}

impl SwcCssCompiler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse_file(
    &self,
    cm: Arc<SourceMap>,
    path: &str,
    source: String,
    config: ParserConfig,
  ) -> Result<TWithDiagnosticArray<Stylesheet>> {
    // let (handler, errors) = self::string_errors::new_handler(cm.clone(), treat_err_as_bug);
    // let result = swc_common::GLOBALS.set(&swc_common::Globals::new(), || op(cm, handler));

    // let fm = cm.load_file(Path::new(path))?;
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);

    let lexer = Lexer::new(SourceFileInput::from(&*fm), config);
    let mut parser = Parser::new(lexer, config);
    let stylesheet = parser.parse_all();
    let diagnostics = parser
      .take_errors()
      .into_iter()
      .flat_map(|error| css_parse_error_to_diagnostic(error, &fm))
      .collect();
    stylesheet
      .map_err(|_| internal_error!("Css parsing failed".to_string()))
      .map(|stylesheet| stylesheet.with_diagnostic(diagnostics))
  }

  pub fn codegen(
    &self,
    cm: Arc<SourceMap>,
    ast: &Stylesheet,
    gen_source_map: SwcCssSourceMapGenConfig,
  ) -> Result<(String, Option<Vec<u8>>)> {
    self.codegen_impl(cm, ast, gen_source_map, false)
  }

  fn codegen_impl(
    &self,
    cm: Arc<SourceMap>,
    ast: &Stylesheet,
    gen_source_map: SwcCssSourceMapGenConfig,
    minify: bool,
  ) -> Result<(String, Option<Vec<u8>>)> {
    let mut output = String::new();
    let mut src_map_buf: Option<Vec<_>> = gen_source_map.enable.then(Vec::new);
    let wr = BasicCssWriter::new(
      &mut output,
      src_map_buf.as_mut(),
      BasicCssWriterConfig::default(),
    );

    let mut gen = CodeGenerator::new(wr, CodegenConfig { minify });
    gen.emit(ast).map_err(|e| internal_error!(e.to_string()))?;

    if let Some(src_map_buf) = &mut src_map_buf {
      let map = cm.build_source_map_with_config(src_map_buf, None, gen_source_map);
      let mut raw_map = Vec::new();
      map
        .to_writer(&mut raw_map)
        .map_err(|e| internal_error!(e.to_string()))?;
      Ok((output, Some(raw_map)))
    } else {
      Ok((output, None))
    }
  }

  pub fn minify(
    &self,
    filename: &str,
    input_source: String,
    input_source_map: Option<rspack_sources::SourceMap>,
    gen_source_map: SwcCssSourceMapGenConfig,
  ) -> Result<rspack_sources::BoxSource> {
    let cm: Arc<SourceMap> = Default::default();
    let parsed = self.parse_file(
      cm.clone(),
      filename,
      input_source.clone(),
      Default::default(),
    )?;
    // ignore errors since css in webpack is tolerant, and diagnostics already reported in parse.
    let (mut ast, _) = parsed.split_into_parts();
    GLOBALS.set(&Globals::default(), || {
      minifier::minify(&mut ast, minifier::options::MinifyOptions::default());
    });
    let (code, source_map) = self.codegen_impl(cm, &ast, gen_source_map, true)?;
    if let Some(source_map) = source_map {
      let source = rspack_sources::SourceMapSource::new(rspack_sources::SourceMapSourceOptions {
        value: code,
        name: filename,
        source_map: rspack_sources::SourceMap::from_slice(&source_map)
          .map_err(|e| internal_error!(e.to_string()))?,
        original_source: Some(input_source),
        inner_source_map: input_source_map,
        remove_original_source: true,
      })
      .boxed();
      Ok(source)
    } else {
      Ok(rspack_sources::RawSource::from(code).boxed())
    }
  }
}

pub struct SwcCssSourceMapGenConfig {
  pub enable: bool,
  pub emit_columns: bool,
  pub inline_sources_content: bool,
}

impl SourceMapGenConfig for SwcCssSourceMapGenConfig {
  fn file_name_to_source(&self, f: &FileName) -> String {
    let f = f.to_string();
    if f.starts_with('<') && f.ends_with('>') {
      f[1..f.len() - 1].to_string()
    } else {
      f
    }
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.emit_columns
  }
}

pub fn css_parse_error_to_diagnostic(error: Error, fm: &SourceFile) -> Vec<Diagnostic> {
  let message = error.message();
  let error = error.into_inner();
  let span: ErrorSpan = error.0.into();
  let traceable_error = rspack_error::TraceableError::from_source_file(
    fm,
    span.start as usize,
    span.end as usize,
    "Css parsing error".to_string(),
    message.to_string(),
  )
  .with_kind(DiagnosticKind::Css)
  // css error tolerate, use `Warning` for recoverable css error
  .with_severity(rspack_error::Severity::Warn);
  //Use this `Error` conversion could avoid eagerly clone source file.
  <Vec<Diagnostic>>::from(rspack_error::Error::TraceableError(traceable_error))
}
