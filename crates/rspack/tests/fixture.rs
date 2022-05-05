mod testing {
  use async_trait::async_trait;
  use rspack::bundler::{BundleContext, BundleOptions, Bundler};
  use rspack::css::plugin::CssSourcePlugin;
  use rspack::traits::plugin::{
    Plugin, PluginLoadHookOutput, PluginResolveHookOutput, PluginTransformHookOutput,
  };
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env;
  use std::fs;
  use std::path::Path;
  use std::sync::atomic::AtomicBool;
  use std::sync::Arc;

  #[tokio::main]
  async fn compile(fixture_path: &str, plugins: Vec<Box<dyn Plugin>>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = dir.join("fixtures").join(fixture_path);
    let pkg_path = fixtures_dir.join("package.json");
    let pkg_content = fs::read_to_string(pkg_path);
    let mut pkg: Value = Value::default();
    if pkg_content.is_ok() {
      pkg = serde_json::from_str(&pkg_content.unwrap()).unwrap();
    }
    // use pkg.rspack.entry if set otherwise use index.js as entry
    let pkg_entry = pkg["rspack"].clone()["entry"].clone();
    let entry = {
      if pkg_entry.is_object() {
        let obj: HashMap<String, String> = serde_json::from_value(pkg_entry).unwrap();
        obj
          .into_iter()
          .map(|(_id, value)| {
            let resolve_path = fixtures_dir.join(value).display().to_string();
            return resolve_path;
          })
          .collect()
      } else {
        let default_entry = fixtures_dir.join("index.js").to_str().unwrap().to_string();
        vec![default_entry]
      }
    };
    let dist = fixtures_dir.join("dist");
    println!("entry: {:?}", entry);
    let mut bundler = Bundler::new(
      BundleOptions {
        entries: entry,
        outdir: dist.to_str().unwrap().to_string(),
        ..Default::default()
      },
      plugins,
    );
    bundler.build().await;
    bundler.write_assets_to_disk();
  }

  #[test]
  fn single_entry() {
    compile("single-entry", vec![])
  }

  #[test]
  fn multi_entry() {
    compile("multi-entry", vec![])
  }

  #[test]
  fn cycle_dep() {
    compile("cycle-dep", vec![])
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

    async fn resolve(
      &self,
      _ctx: &BundleContext,
      _id: &str,
      _importer: Option<&str>,
    ) -> PluginResolveHookOutput {
      self
        .call_resolve
        .store(true, std::sync::atomic::Ordering::SeqCst);
      None
    }

    #[inline]
    async fn load(&self, _ctx: &BundleContext, _id: &str) -> PluginLoadHookOutput {
      self
        .call_load
        .store(true, std::sync::atomic::Ordering::SeqCst);
      None
    }

    #[inline]
    fn transform(
      &self,
      _ctx: &BundleContext,
      ast: swc_ecma_ast::Module,
    ) -> PluginTransformHookOutput {
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
    compile("dynamic-import", vec![])
  }

  #[test]
  #[ignore]
  fn basic_css() {
    compile("basic-css", vec![])
  }

  #[test]
  #[ignore = "not support npm yet"]
  fn npm() {
    compile("npm", vec![])
  }

  #[test]
  fn cjs() {
    compile("cjs", vec![])
  }

  #[test]
  fn css_bundle_test() {
    let css_plugin: CssSourcePlugin = std::default::Default::default();
    compile("css", vec![Box::new(css_plugin)])
  }
}
