// mod js_module;
// pub use js_module::*;
pub mod module;
pub mod plugin;
pub mod visitors;

use once_cell::sync::Lazy;

use rspack_core::{ErrorSpan, PATH_START_BYTE_POS_MAP};
use rspack_error::{
  Diagnostic, DiagnosticKind, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use swc_common::{input::SourceFileInput, sync::Lrc, FileName, FilePathMapping, SourceMap};

use std::sync::Arc;

use swc_css::parser::{lexer::Lexer, parser::ParserConfig};
use swc_css::{ast::Stylesheet, parser::parser::Parser};
use swc_css::{
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::error::Error,
};

pub use plugin::CssPlugin;

static SWC_COMPILER: Lazy<Arc<SwcCssCompiler>> = Lazy::new(|| Arc::new(SwcCssCompiler::new()));

#[derive(Default)]
pub struct SwcCssCompiler {}

static CM: Lazy<Lrc<SourceMap>> = Lazy::new(|| Lrc::new(SourceMap::new(FilePathMapping::empty())));

impl SwcCssCompiler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse_file(&self, path: &str, source: String) -> Result<TWithDiagnosticArray<Stylesheet>> {
    let config: ParserConfig = Default::default();
    let cm = CM.clone();
    // let (handler, errors) = self::string_errors::new_handler(cm.clone(), treat_err_as_bug);
    // let result = swc_common::GLOBALS.set(&swc_common::Globals::new(), || op(cm, handler));

    // let fm = cm.load_file(Path::new(path))?;
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);

    PATH_START_BYTE_POS_MAP.insert(path.to_string(), fm.start_pos.0);
    let lexer = Lexer::new(SourceFileInput::from(&*fm), config);
    let mut parser = Parser::new(lexer, config);
    let stylesheet = parser.parse_all();
    let diagnostics = parser
      .take_errors()
      .into_iter()
      .flat_map(|error| css_parse_error_to_diagnostic(error, path))
      .collect();
    stylesheet
      .map_err(|_| rspack_error::Error::InternalError("Css parsing failed".to_string()))
      .map(|stylesheet| stylesheet.with_diagnostic(diagnostics))
  }

  pub fn codegen(&self, ast: &Stylesheet) -> String {
    let _config: CodegenConfig = CodegenConfig { minify: false };
    let mut output = String::new();
    let wr = BasicCssWriter::new(
      &mut output,
      None, // Some(&mut src_map_buf),
      BasicCssWriterConfig::default(),
    );

    let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

    gen.emit(ast).unwrap();

    output
  }
}

pub fn css_parse_error_to_diagnostic(error: Error, path: &str) -> Vec<Diagnostic> {
  let message = error.message();
  let error = error.into_inner();
  let span: ErrorSpan = error.0.into();
  let traceable_error = rspack_error::TraceableError::from_path(
    path.to_string(),
    span.start as usize,
    span.end as usize,
    "Css parsing error".to_string(),
    message.to_string(),
  )
  .with_kind(DiagnosticKind::Css);
  //Use this `Error` convertion could avoid eagerly clone source file.
  <Vec<Diagnostic>>::from(rspack_error::Error::TraceableError(traceable_error))
}
