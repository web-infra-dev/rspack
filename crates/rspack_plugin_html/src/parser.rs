use rspack_core::{ErrorSpan, PATH_START_BYTE_POS_MAP};
use rspack_error::{
  Diagnostic, DiagnosticKind, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use swc_common::{sync::Lrc, FileName, FilePathMapping, SourceMap};
use swc_html::{
  ast::Document,
  codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{error::Error, parse_file_as_document, parser::ParserConfig},
};

#[derive(Default)]
pub struct HtmlCompiler {}

impl HtmlCompiler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse_file(&self, path: &str, source: String) -> Result<TWithDiagnosticArray<Document>> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);

    PATH_START_BYTE_POS_MAP.insert(path.to_string(), fm.start_pos.0);

    let mut errors = vec![];
    let document = parse_file_as_document(fm.as_ref(), ParserConfig::default(), &mut errors);
    let diagnostics: Vec<rspack_error::Diagnostic> = errors
      .into_iter()
      .flat_map(|error| <Vec<Diagnostic>>::from(html_parse_error_to_traceable_error(error, path)))
      .collect();
    document
      .map(|doc| doc.with_diagnostic(diagnostics))
      .map_err(|e| html_parse_error_to_traceable_error(e, path))
  }

  pub fn codegen(&self, ast: &Document) -> anyhow::Result<String> {
    let writer_config = BasicHtmlWriterConfig::default();
    let codegen_config = CodegenConfig::default();

    let mut output = String::new();
    let wr = BasicHtmlWriter::new(&mut output, None, writer_config);
    let mut gen = CodeGenerator::new(wr, codegen_config);

    gen.emit(&ast).map_err(|e| anyhow::format_err!(e))?;
    Ok(output)
  }
}

pub fn html_parse_error_to_traceable_error(error: Error, path: &str) -> rspack_error::Error {
  let message = error.message();
  let error = error.into_inner();
  let span: ErrorSpan = error.0.into();
  let traceable_error = rspack_error::TraceableError::from_path(
    path.to_string(),
    span.start as usize,
    span.end as usize,
    "HTML parsing error".to_string(),
    message.to_string(),
  )
  .with_kind(DiagnosticKind::Html);
  //Use this `Error` convertion could avoid eagerly clone source file.
  rspack_error::Error::TraceableError(traceable_error)
}
