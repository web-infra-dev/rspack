use hashbrown::HashMap;
use hrx_parser::Entry;

#[test]
fn valid() {
  let file = include_str!("./cases/fixtures.hrx");
  for unit in normalize(file)
    .into_iter()
    .filter(|unit| !unit.path.starts_with('-'))
  {
    insta::with_settings!({sort_maps => false, snapshot_path => "cases", prepend_module_to_snapshot => false, snapshot_suffix => ""}, {
        insta::assert_snapshot!(unit.path, unit.content);
    });
  }
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
