use hashbrown::HashMap;
use hrx_parser::Entry;
use rspack_error::TWithDiagnosticArray;
use swc_common::{FileName, FilePathMapping, SourceFile, SourceMap};
use swc_css::{
  ast::{self, Stylesheet},
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{parse_file, parser::ParserConfig},
  visit::VisitMutWith,
};

use crate::SWC_COMPILER;

use super::{option::PxToRemOption, px_to_rem::px_to_rem};

#[test]
fn valid() {
  let file = include_str!("./cases/fixtures.hrx");
  for unit in normalize(file)
    .into_iter()
    .filter(|unit| !unit.path.starts_with('-'))
  {
    insta::with_settings!({sort_maps => false, snapshot_path => "cases", prepend_module_to_snapshot => false, snapshot_suffix => ""}, {
      let config = unit.meta_data.get("config");
      dbg!(&unit.path);
      insta::assert_snapshot!(unit.path, transform(&unit.content, config));
    });
  }
}

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

#[derive(Debug)]
struct TestUnit {
  path: String,
  content: String,
  meta_data: HashMap<String, String>,
}

fn normalize(source: &str) -> Vec<TestUnit> {
  let mut res = vec![];
  let archive = hrx_parser::parse(source).unwrap();
  let mut i = 0;
  while i < archive.entries.len() {
    let entry = &archive.entries[i];
    let mut unit = if let Some(unit) = convert_entry_to_unit(entry) {
      unit
    } else {
      continue;
    };
    i += 1;
    while i < archive.entries.len() {
      let another_entry = &archive.entries[i];
      if !another_entry.path().starts_with('.') {
        break;
      }
      if let Some(another_unit) = convert_entry_to_unit(another_entry) {
        unit
          .meta_data
          .insert(another_unit.path[1..].to_string(), another_unit.content);
      }
      i += 1;
    }
    res.push(unit);
  }
  res
}

fn convert_entry_to_unit(entry: &Entry) -> Option<TestUnit> {
  entry.content().map(|content| {
    let path = entry.path();
    TestUnit {
      path,
      content,
      meta_data: HashMap::default(),
    }
  })
}
