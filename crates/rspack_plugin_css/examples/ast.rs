use std::time::Instant;

use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_css::{
  ast::Stylesheet,
  codegen::writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  parser::{parse_file, parser::ParserConfig},
  visit::VisitMutWith,
};

use rspack_plugin_css::pxtorem::{option::PxToRemOption, px_to_rem::px_to_rem};

fn transform(source: &str, config_file: Option<&String>) -> String {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom("test.css".to_owned()), source.to_owned());
  for _ in 0..100 {
    let start = Instant::now();
    let mut stylesheet =
      parse_file::<Stylesheet>(&fm, ParserConfig::default(), &mut vec![]).unwrap();
    let mut output = String::new();
    let _wr = BasicCssWriter::new(
      &mut output,
      None, // Some(&mut src_map_buf),
      BasicCssWriterConfig::default(),
    );
    let config: PxToRemOption = config_file
      .map(|file| serde_json::from_str(file).unwrap())
      .unwrap_or_default();
    // let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

    stylesheet.visit_mut_with(&mut px_to_rem(config));
    println!("transform: {:?}", start.elapsed());
  }
  // gen.emit(&stylesheet).unwrap();

  String::new()
}
fn main() {
  let source = "";
  let config = Some(
    r#"
  {
    "propList": ["*"]
  }

    "#
    .to_string(),
  );

  let start = Instant::now();
  transform(source, config.as_ref());
  println!("{:?}", start.elapsed());
}
