use anyhow::Ok;
use swc_common::{sync::Lrc, FileName, FilePathMapping, SourceMap};
use swc_html::{
  ast::Document,
  codegen::{
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{parse_file_as_document, parser::ParserConfig},
};

#[derive(Default)]
pub struct HtmlCompiler {}

impl HtmlCompiler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse_file(&self, path: &str, source: String) -> anyhow::Result<Document> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);
    let mut errors = vec![];
    let document = parse_file_as_document(
      &fm,
      ParserConfig {
        ..Default::default()
      },
      &mut errors,
    )
    .map_err(|e| anyhow::format_err!(e.message()))?;

    // for err in errors {
    //   eprintln!("{:?}", err);
    // }

    Ok(document)
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
