mod utils;

use async_trait::async_trait;
use rspack::bundler::{BundleContext, BundleOptions};
use utils::{compile, compile_with_options};

use rspack_core::{LoadArgs, Loader, ResolveArgs};
use rspack_core::{
  Plugin, PluginLoadHookOutput, PluginResolveHookOutput, PluginTransformAstHookOutput,
};

use rspack_swc::swc_ecma_ast;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[test]
fn single_entry() {
  compile("single-entry", vec![]);
}

#[test]
fn multi_entry() {
  compile("multi-entry", vec![]);
}

#[test]
fn cycle_dep() {
  compile("cycle-dep", vec![]);
}

#[derive(Debug)]
struct TestPlugin {
  call_resolve: Arc<AtomicBool>,
  call_load: Arc<AtomicBool>,
  call_transform: Arc<AtomicBool>,
}

#[async_trait]
impl Plugin for TestPlugin {
  fn name(&self) -> &'static str {
    "rspack_test"
  }

  async fn resolve(&self, _ctx: &BundleContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    self
      .call_resolve
      .store(true, std::sync::atomic::Ordering::SeqCst);
    None
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    self
      .call_load
      .store(true, std::sync::atomic::Ordering::SeqCst);
    None
  }

  #[inline]
  fn transform_ast(
    &self,
    _ctx: &BundleContext,
    _path: &Path,
    ast: swc_ecma_ast::Module,
  ) -> PluginTransformAstHookOutput {
    self
      .call_transform
      .store(true, std::sync::atomic::Ordering::SeqCst);
    ast
  }
}

#[test]
fn plugin_test() {
  let call_resolve: Arc<AtomicBool> = Default::default();
  let call_load: Arc<AtomicBool> = Default::default();
  let call_transform: Arc<AtomicBool> = Default::default();
  let test_plugin = Box::new(TestPlugin {
    call_resolve: call_resolve.clone(),
    call_load: call_load.clone(),
    call_transform: call_transform.clone(),
  });
  compile("single-entry", vec![test_plugin]);
  assert!(call_load.load(std::sync::atomic::Ordering::SeqCst));
  assert!(call_resolve.load(std::sync::atomic::Ordering::SeqCst));
  assert!(call_transform.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn dynamic_import() {
  compile("dynamic-import", vec![]);
}

#[test]
fn basic_css() {
  let bundler = compile("basic-css", vec![]);
  println!(
    "plugin_name -> \n {:#?}",
    bundler
      .plugin_driver
      .plugins
      .iter()
      .map(|x| x.name().to_string())
      .collect::<Vec<String>>()
  );
  assert!(bundler
    .plugin_driver
    .plugins
    .iter()
    .find(|plugin| plugin.name() == rspack_plugin_stylesource::plugin::PLUGIN_NAME)
    .is_some())
}

#[test]
#[ignore = "not support npm yet"]
fn npm() {
  compile("npm", vec![]);
}

#[test]
fn cjs() {
  compile("cjs", vec![]);
}

#[test]
fn css_bundle_test() {
  compile_with_options(
    "css",
    BundleOptions {
      loader: HashMap::from_iter([
        ("css".to_string(), Loader::Css),
        ("less".to_string(), Loader::Less),
        ("sass".to_string(), Loader::Sass),
        ("scss".to_string(), Loader::Sass),
        ("svg".to_string(), Loader::DataURI),
      ]),
      ..Default::default()
    },
    vec![],
  );

  pub fn path_resolve(path: &str) -> String {
    let work_cwd = env!("CARGO_MANIFEST_DIR");
    let os_work_cwd = OsString::from(work_cwd);
    Path::new(&os_work_cwd)
      .join(path)
      .into_os_string()
      .into_string()
      .unwrap()
  }

  let _dist_css_file1 = path_resolve("fixtures/css/dist/index.css");
  let _dist_css_file2 = path_resolve("fixtures/css/dist/liba.css");
  // FIXME: The output filename of chunk is not stable now, should not rely on it.
  // assert_eq!(Path::new(dist_css_file1.as_str()).exists(), true);
  // assert_eq!(Path::new(dist_css_file2.as_str()).exists(), true);
}

#[test]
fn disable_code_splitting() {
  let bundler = compile_with_options(
    "basic",
    BundleOptions {
      code_splitting: None,
      ..Default::default()
    },
    vec![],
  );
  let chunk_len = bundler.bundle.context.assets.lock().unwrap().len();
  assert_eq!(chunk_len, 2);
}

#[test]
fn enable_code_splitting() {
  let bundler = compile_with_options(
    "basic",
    BundleOptions {
      code_splitting: Some(Default::default()),
      ..Default::default()
    },
    vec![],
  );
  let chunk_len = bundler.bundle.context.assets.lock().unwrap().len();
  assert_eq!(chunk_len, 3);
}

#[test]
fn basic_ts() {
  compile("basic-ts", vec![]);
}

#[test]
fn svgr() {
  compile("svgr", vec![]);
}

#[test]
fn splitting() {
  compile("code-splitting", vec![]);
}

#[test]
fn loader() {
  compile_with_options(
    "loader",
    BundleOptions {
      loader: vec![
        ("svg".to_string(), Loader::DataURI),
        // Json is supported by default
        // ("json".to_string(), Loader::Json),
        ("txt".to_string(), Loader::Text),
      ]
      .into_iter()
      .collect(),
      ..Default::default()
    },
    vec![],
  );
}

#[test]
fn stack_overflow_mockjs() {
  compile("stack_overflow_mockjs", vec![]);
}
