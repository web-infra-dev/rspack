use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_css::{
  ast::Stylesheet,
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{parse_file, parser::ParserConfig},
  visit::VisitMutWith,
};

use rspack_plugin_css::pxtorem::{option::PxToRemOption, px_to_rem::px_to_rem};

fn transform(source: &str, config_file: Option<&String>) -> String {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom("test.css".to_owned()), source.to_owned());
  let mut stylesheet = parse_file::<Stylesheet>(&fm, ParserConfig::default(), &mut vec![]).unwrap();

  let mut output = String::new();
  let wr = BasicCssWriter::new(
    &mut output,
    None, // Some(&mut src_map_buf),
    BasicCssWriterConfig::default(),
  );
  let config: PxToRemOption = config_file
    .map(|file| serde_json::from_str(file).unwrap())
    .unwrap_or_default();
  dbg!(&config);
  let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

  stylesheet.visit_mut_with(&mut px_to_rem(config));
  gen.emit(&stylesheet).unwrap();

  output
}
fn main() {
  let source = r#"
h1 {
  margin: 0 0 20px;
  font-size: 2rem;
  line-height: 1.2;
  letter-spacing: 0.0625rem;
}
  "#;
  let config = Some(
    r#"
{
}
    
  "#
    .to_string(),
  );

  println!("{}", transform(source, config.as_ref()));
}
