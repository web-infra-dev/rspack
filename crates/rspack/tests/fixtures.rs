use std::path::PathBuf;

use insta::Settings;
use rspack_binding_options::RawOptions;
use rspack_core::tree_shaking::visitor::TreeShakingResult;
use rspack_core::Compilation;
use rspack_core::CompilerOptions;
use rspack_symbol::Symbol;
use rspack_test::{read_test_config_and_normalize, test_fixture, test_options::RawOptionsExt};
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;
use ustr::Ustr;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path);
}

#[tokio::main]
async fn run(context: PathBuf) {
  let mut options = read_test_config_and_normalize(&context);
  options.__emit_error = true;
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.run().await.unwrap();
}

#[fixture("../../examples/*")]
fn example(fixture_path: PathBuf) {
  run(fixture_path);
}

#[test]
fn eaaaaxample() {
  let fixture_path: PathBuf =
    PathBuf::from("/Users/bytedance/rspack/packages/rspack/tests/cases/resolve/dep-condition");
  run(fixture_path);
}

#[fixture("tests/tree-shaking/*")]
fn tree_shaking(fixture_path: PathBuf) {
  tree_shaking_test(&fixture_path).unwrap()
}

#[tokio::main]
pub async fn tree_shaking_test(fixture_path: &PathBuf) -> rspack_error::Result<()> {
  let options: CompilerOptions = RawOptions::from_fixture(fixture_path).to_compiler_options();
  let mut compiler = rspack::rspack(options, Default::default());

  let stats = compiler
    .build()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));
  let snapshot = get_used_snapshot(stats.compilation).await;
  let mut settings = Settings::clone_current();
  settings.remove_snapshot_suffix();
  settings.set_snapshot_path(fixture_path);
  let dirname = fixture_path
    .components()
    .last()
    .unwrap()
    .as_os_str()
    .to_str()
    .unwrap()
    .to_owned();
  settings.bind(|| {
    insta::assert_snapshot!(dirname.as_str(), snapshot, dirname.as_str());
  });
  Ok(())
}

pub async fn get_used_snapshot(compilation: &Compilation) -> String {
  let mut ret = Vec::new();
  let common_prefix_string = common_prefix(
    compilation
      .tree_shaking_result
      .keys()
      .cloned()
      .collect::<Vec<_>>(),
  );
  for (key, result) in compilation.tree_shaking_result.iter() {
    let mut single_snapshot = format!(
      "\n// <-- relative path: {} -->\n\n",
      &key.as_str()[common_prefix_string.len()..]
    );
    single_snapshot += &tokio::fs::read_to_string(key.as_str()).await.unwrap();
    single_snapshot += "// <-- unused-export: --> \n";
    let unused_symbol_list = get_unused_list(result, &compilation.used_symbol);
    single_snapshot += &format!("{:#?}", unused_symbol_list);
    ret.push((key, single_snapshot));
  }
  ret.sort_by(|a, b| a.0.cmp(b.0));
  ret
    .into_iter()
    .map(|case| case.1)
    .fold(String::new(), |pre, cur| pre + &cur)
}

fn common_prefix(str_list: Vec<Ustr>) -> String {
  let ret = str_list.iter().fold(str_list[0].as_str(), |pre, cur| {
    common_prefix_of_two(pre, cur.as_str())
  });
  ret.to_string()
}

fn common_prefix_of_two<'a>(a: &'a str, b: &'a str) -> &'a str {
  let min_len = a.len().min(b.len());
  let mut max_common_len = 0;
  for (a, b) in a.chars().take(min_len).zip(b.chars().take(min_len)) {
    if a == b {
      max_common_len += 1;
    } else {
      break;
    }
  }
  &a[0..max_common_len]
}

fn get_unused_list(
  result: &TreeShakingResult,
  used_set: &hashbrown::HashSet<Symbol>,
) -> Vec<String> {
  let mut unused_symbol = vec![];
  for (_, sym_ref) in result.export_map.iter() {
    if let rspack_core::tree_shaking::visitor::SymbolRef::Direct(sym) = sym_ref {
      if !used_set.contains(sym) {
        unused_symbol.push(sym.id().atom.to_string());
      }
    }
  }
  unused_symbol.sort();
  unused_symbol
}
