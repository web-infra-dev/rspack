use rspack_core::ErrorSpan;
use rspack_error::{error, DiagnosticKind, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::{sync::Lrc, FileName, FilePathMapping, SourceFile, SourceMap, GLOBALS};
use swc_html::{
  ast::Document,
  codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{error::Error, parse_file_as_document, parser::ParserConfig},
};
use swc_html_minifier::minify_document;
pub use swc_html_minifier::option::MinifyOptions;

use crate::config::HtmlRspackPluginOptions;

pub struct HtmlCompiler<'a> {
  config: &'a HtmlRspackPluginOptions,
}

impl<'a> HtmlCompiler<'a> {
  pub fn new(config: &'a HtmlRspackPluginOptions) -> Self {
    Self { config }
  }

  pub fn parse_file(&self, path: &str, source: String) -> Result<TWithDiagnosticArray<Document>> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);

    let mut errors = vec![];
    let document = parse_file_as_document(fm.as_ref(), ParserConfig::default(), &mut errors);
    let diagnostics: Vec<rspack_error::Diagnostic> = errors
      .into_iter()
      .flat_map(|error| vec![html_parse_error_to_traceable_error(error, &fm).into()])
      .collect();
    document
      .map(|doc| doc.with_diagnostic(diagnostics))
      .map_err(|e| html_parse_error_to_traceable_error(e, &fm))
  }

  pub fn codegen(&self, ast: &mut Document) -> Result<String> {
    let writer_config = BasicHtmlWriterConfig::default();
    let codegen_config = CodegenConfig {
      minify: self.config.minify,
      ..Default::default()
    };
    if self.config.minify {
      // Minify can't leak to user land because it doesn't implement `ToNapiValue` Trait
      GLOBALS.set(&Default::default(), || {
        minify_document(ast, &MinifyOptions::default());
      })
    }

    let mut output = String::new();
    let wr = BasicHtmlWriter::new(&mut output, None, writer_config);
    let mut gen = CodeGenerator::new(wr, codegen_config);

    gen.emit(ast).map_err(|e| error!(e.to_string()))?;
    Ok(output)
  }
}

pub fn html_parse_error_to_traceable_error(error: Error, fm: &SourceFile) -> rspack_error::Error {
  let message = error.message();
  let error = error.into_inner();
  let span: ErrorSpan = error.0.into();
  let traceable_error = rspack_error::TraceableError::from_source_file(
    fm,
    span.start as usize,
    span.end as usize,
    "HTML parsing error".to_string(),
    message.to_string(),
  )
  .with_kind(DiagnosticKind::Html);
  //Use this `Error` conversion could avoid eagerly clone source file.
  traceable_error.into()
}
