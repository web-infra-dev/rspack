#![feature(box_patterns)]
// mod js_module;
// pub use js_module::*;
pub mod module;
pub mod plugin;
pub mod pxtorem;
pub mod visitors;

use once_cell::sync::Lazy;

use rspack_core::rspack_sources::Source;
use rspack_core::{ErrorSpan, PATH_START_BYTE_POS_MAP};
use rspack_error::{
  Diagnostic, DiagnosticKind, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use swc_common::source_map::SourceMapGenConfig;
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

  pub fn codegen(
    &self,
    ast: &Stylesheet,
    orignal_source: Option<&dyn Source>,
  ) -> Result<(String, Option<String>)> {
    let mut output = String::new();
    let mut src_map_buf = orignal_source.is_some().then(Vec::new);
    let wr = BasicCssWriter::new(
      &mut output,
      src_map_buf.as_mut(),
      BasicCssWriterConfig::default(),
    );

    let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

    gen.emit(ast).unwrap();

    if let Some(src_map_buf) = &mut src_map_buf {
      let map = CM.build_source_map_with_config(src_map_buf, None, SwcCssSourceMapGenConfig);
      let mut raw_map = Vec::new();
      map
        .to_writer(&mut raw_map)
        .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
      let map_string = String::from_utf8_lossy(&raw_map).to_string();
      Ok((output, Some(map_string)))
    } else {
      Ok((output, None))
    }
  }
}

struct SwcCssSourceMapGenConfig;

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
    true
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
