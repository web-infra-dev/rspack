use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_rst::{helper::make_relative_from, rst::RstBuilder};
use insta::{assert_snapshot, Settings};
use itertools::Itertools;
use rspack_binding_options::RawOptions;
use rspack_core::{BoxPlugin, Compiler, CompilerOptions};
use rspack_fs::AsyncNativeFileSystem;
use rspack_tracing::enable_tracing_by_env;

use crate::{eval_raw::evaluate_to_json, test_config::TestConfig};

pub type MutTestOptionsFn = dyn FnMut(&mut Vec<BoxPlugin>, &mut CompilerOptions);

pub fn apply_from_fixture(fixture_path: &Path) -> (CompilerOptions, Vec<BoxPlugin>) {
  let js_config = fixture_path.join("test.config.js");
  if js_config.exists() {
    let raw = evaluate_to_json(&js_config);
    let raw: RawOptions = serde_json::from_slice(&raw).expect("ok");
    let compiler_options = raw.try_into().expect("should be ok");
    return (compiler_options, Vec::new());
  }
  let json_config = fixture_path.join("test.config.json");
  let test_config = TestConfig::from_config_path(&json_config);
  test_config.apply(fixture_path.to_path_buf())
}
#[tokio::main]
pub async fn test_fixture_html(fixture_path: &Path) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(
    fixture_path,
    &|s| s.ends_with(".html"),
    Box::new(|_, _| {}),
    None,
  )
  .await
}
#[tokio::main]
pub async fn test_fixture_js(fixture_path: &Path) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(
    fixture_path,
    &|s| s.ends_with(".js") && !s.contains("runtime.js"),
    Box::new(|_, _| {}),
    None,
  )
  .await
}
#[tokio::main]
pub async fn test_fixture_css(fixture_path: &Path) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(
    fixture_path,
    &|s| s.ends_with(".css"),
    Box::new(|_, _| {}),
    None,
  )
  .await
}
#[tokio::main]
pub async fn test_fixture_css_modules(fixture_path: &Path) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(
    fixture_path,
    &|s| s.ends_with(".css") || (s.ends_with(".js") && !s.contains("runtime.js")),
    Box::new(|_, _| {}),
    None,
  )
  .await
}
#[tokio::main]
pub async fn test_fixture_insta(
  fixture_path: &Path,
  stats_filter: &dyn Fn(&str) -> bool,
  mut_settings: Box<MutTestOptionsFn>,
) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(fixture_path, stats_filter, mut_settings, None).await
}

#[tokio::main]
pub async fn test_fixture(
  fixture_path: &Path,
  mut_settings: Box<MutTestOptionsFn>,
  snapshot_name: Option<String>,
) -> Compiler<AsyncNativeFileSystem> {
  test_fixture_share(
    fixture_path,
    &|s| s.ends_with(".js") && !s.contains("runtime.js"),
    mut_settings,
    snapshot_name,
  )
  .await
}

/// You can mutate the `Settings` of insta and  `CompilerOptions` of rspack via `mut_settings` mut closure
pub async fn test_fixture_share(
  fixture_path: &Path,
  stats_filter: &dyn Fn(&str) -> bool,
  mut mut_settings: Box<MutTestOptionsFn>,
  snapshot_name: Option<String>,
) -> Compiler<AsyncNativeFileSystem> {
  enable_tracing_by_env(&std::env::var("TRACE").ok().unwrap_or_default(), "stdout");

  let snapshot_name = snapshot_name.unwrap_or("output".to_string());

  let mut settings = Settings::clone_current();
  settings.set_snapshot_path(Path::new(fixture_path).join("snapshot"));
  settings.set_omit_expression(true);
  settings.set_prepend_module_to_snapshot(false);

  let (mut options, mut plugins) = apply_from_fixture(fixture_path);

  mut_settings(&mut plugins, &mut options);

  // clean output
  if options.output.path.exists() {
    std::fs::remove_dir_all(&options.output.path).expect("should remove output");
  }
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);

  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixture {fixture_path:?}, {e:#?}"));
  let assets = compiler.compilation.assets();

  let content = assets
    .iter()
    .filter_map(|(filename, asset)| {
      if stats_filter(filename) {
        println!("{:?}", settings.description());
        let content = asset
          .get_source()
          .map(|x| x.source().to_string())
          .unwrap_or(String::from("this is an empty asset"));
        let tag = Path::new(filename)
          .extension()
          .map(|x| x.to_string_lossy())
          .unwrap_or(std::borrow::Cow::Borrowed("txt"));
        Some(format!("```{tag} title={filename}\n{content}\n```"))
      } else {
        None
      }
    })
    .sorted()
    .join("\n\n");
  settings.bind(|| {
    assert_snapshot!(snapshot_name.as_str(), content);
  });
  let stats = compiler.compilation.get_stats();
  let warnings = stats.get_warnings();
  let errors = stats.get_errors();
  if !warnings.is_empty() && errors.is_empty() {
    println!(
      "Warning to compile in fixture {:?}, warnings: {:?}",
      fixture_path,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    )
  }
  if !errors.is_empty() {
    panic!(
      "Failed to compile in fixture {:?}, errors: {:?}",
      fixture_path,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    );
  }
  compiler
}

fn read_dir_reverse(path: &PathBuf) -> Vec<String> {
  let mut result = vec![];
  if let Ok(changed_dir) = std::fs::read_dir(path) {
    changed_dir.for_each(|entry| {
      let entry = entry.expect("TODO:");
      if entry.path().is_file() {
        result.push(entry.path().as_os_str().to_string_lossy().to_string());
      }
      if entry.path().is_dir() {
        result.extend(read_dir_reverse(&entry.path()))
      }
    })
  }
  result
}

enum FsOptionEnum {
  Change(Vec<u8>),
  Create,
  Remove(Vec<u8>),
}

#[tokio::main]
pub async fn test_rebuild_fixture(
  fixture_path: &Path,
  cb: Option<Box<dyn FnOnce(Compiler<AsyncNativeFileSystem>)>>,
) {
  enable_tracing_by_env(&std::env::var("TRACE").ok().unwrap_or_default(), "stdout");

  let (options, plugins) = apply_from_fixture(fixture_path);
  // clean output
  if options.output.path.exists() {
    std::fs::remove_dir_all(&options.output.path).expect("should remove output");
  }
  let mut compiler = Compiler::new(options, plugins, AsyncNativeFileSystem);
  compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("failed to compile in fixture {fixture_path:?}, {e:#?}"));

  let mut files_map: HashMap<String, FsOptionEnum> = HashMap::new();
  let changed_files: HashSet<String> =
    HashSet::from_iter(read_dir_reverse(&fixture_path.join("changed")).into_iter());
  let removed_files: HashSet<String> =
    HashSet::from_iter(read_dir_reverse(&fixture_path.join("removed")).into_iter());
  let created_files: HashSet<String> =
    HashSet::from_iter(read_dir_reverse(&fixture_path.join("created")).into_iter());

  let mut created_files: HashSet<String> = created_files
    .iter()
    .map(|file_path| {
      let target_path = file_path.replacen("created/", "", 1);
      files_map.insert(target_path.clone(), FsOptionEnum::Create);
      let created_context = std::fs::read(file_path.clone());
      std::fs::write(
        target_path.clone(),
        created_context.expect("changed file do not read"),
      )
      .expect("TODO: panic message");
      target_path
    })
    .collect();
  let changed_files: HashSet<String> = changed_files
    .iter()
    .map(|file_path| {
      let target_path = file_path.replacen("changed/", "", 1);
      let target_context = std::fs::read(target_path.clone());
      files_map.insert(
        target_path.clone(),
        FsOptionEnum::Change(target_context.expect("change file not found")),
      );
      let new_content = std::fs::read(file_path);
      std::fs::write(
        target_path.clone(),
        new_content.expect("changed file do not read"),
      )
      .expect("TODO: panic message");
      target_path
    })
    .collect();
  let removed_files: HashSet<String> = removed_files
    .iter()
    .map(|file_path| {
      let target_path = file_path.replacen("removed/", "", 1);
      let target_context = std::fs::read(target_path.clone());
      files_map.insert(
        target_path.clone(),
        FsOptionEnum::Remove(target_context.expect("change file not found")),
      );
      target_path
    })
    .collect();
  created_files.extend(changed_files.into_iter());
  let changed_files = created_files;
  compiler
    .rebuild(changed_files, removed_files)
    .await
    .unwrap_or_else(|e| panic!("failed to rebuild in fixture {fixture_path:?}, {e:#?}"));

  files_map
    .iter()
    .for_each(|(old_path, old_content)| match old_content {
      FsOptionEnum::Create => {
        std::fs::remove_file(old_path).expect("TODO: panic message");
      }
      FsOptionEnum::Change(old_content) | FsOptionEnum::Remove(old_content) => {
        std::fs::write(old_path.clone(), old_content).expect("TODO: panic message");
      }
    });
  let stats = compiler.compilation.get_stats();
  let output_name = make_relative_from(&compiler.options.output.path, fixture_path);
  let rst = RstBuilder::default()
    .fixture(PathBuf::from(fixture_path))
    .actual(output_name)
    .build()
    .expect("TODO:");

  let errors = stats.get_errors();
  if !errors.is_empty() {
    panic!(
      "Failed to compile in fixture {:?}, errors: {:?}",
      fixture_path,
      stats
        .emit_diagnostics_string(true)
        .expect("failed to emit diagnostics to string")
    );
  }
  rst.assert();

  if let Some(cb) = cb {
    cb(compiler);
  }
}
