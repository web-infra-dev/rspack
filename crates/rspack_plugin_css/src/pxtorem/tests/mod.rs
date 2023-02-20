use std::fmt::Write;

use hrx_parser::Entry;
use rustc_hash::FxHashMap as HashMap;
use swc_core::common::{FileName, FilePathMapping, SourceMap};
use swc_core::css::{
  ast::Stylesheet,
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  parser::{parse_file, parser::ParserConfig},
  visit::VisitMutWith,
};

use super::{options::PxToRemOptions, px_to_rem::px_to_rem};

#[test]
fn valid() {
  let file = include_str!("./cases/fixtures.hrx");
  for unit in normalize(file)
    .into_iter()
    .filter(|unit| !unit.path.starts_with('-'))
  {
    insta::with_settings!({sort_maps => false, snapshot_path => "cases", prepend_module_to_snapshot => false, snapshot_suffix => ""}, {
      let config = unit.meta_data.get("config");
      let expected = unit.meta_data.get("expected").cloned().unwrap_or_default();
      let snapshot_result = get_snapshot_result(&unit.content, &expected, &transform(&unit.content, config));
      let snapshot_path = unit.path.replace(' ', "_");
      insta::assert_snapshot!(snapshot_path.clone(), snapshot_result, &snapshot_path);
    });
  }
}

fn transform(source: &str, config_file: Option<&String>) -> String {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom("test.css".to_owned()), source.to_owned());
  let mut stylesheet =
    parse_file::<Stylesheet>(&fm, ParserConfig::default(), &mut vec![]).expect("TODO:");

  let mut output = String::new();
  let wr = BasicCssWriter::new(
    &mut output,
    None, // Some(&mut src_map_buf),
    BasicCssWriterConfig::default(),
  );
  let config: PxToRemOptions = config_file
    .map(|file| serde_json::from_str(file).expect("TODO:"))
    .unwrap_or_default();
  let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

  stylesheet.visit_mut_with(&mut px_to_rem(config));
  gen.emit(&stylesheet).expect("TODO:");

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
  let archive = hrx_parser::parse(source).expect("TODO:");
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

fn get_snapshot_result(input: &str, expected: &str, actual: &str) -> String {
  let mut result = String::new();
  writeln!(result, "# Input").expect("should success");
  writeln!(result, "{input}").expect("should success");
  writeln!(result, "# Expected").expect("should success");
  writeln!(result, "{expected}").expect("should success");
  writeln!(result, "# Actual").expect("should success");
  writeln!(result, "{actual}").expect("should success");
  result
}
