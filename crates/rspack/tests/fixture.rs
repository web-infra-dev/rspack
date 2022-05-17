mod testing {
  use async_trait::async_trait;
  use rspack::bundler::{BundleContext, BundleOptions, Bundler};
  use rspack_core::{Loader, ResolveOption};
  use rspack_core::{
    Plugin, PluginLoadHookOutput, PluginResolveHookOutput, PluginTransformHookOutput,
  };

  use rspack_swc::swc_ecma_ast;
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env;
  use std::fs;
  use std::path::Path;
  use std::sync::atomic::AtomicBool;
  use std::sync::Arc;
  use std::sync::Once;

  static INIT: Once = Once::new();

  fn compile(fixture_path: &str, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
    INIT.call_once(|| {
      let default_panic = std::panic::take_hook();
      std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
      }));
    });
    compile_with_options(fixture_path, Default::default(), plugins)
  }

  fn compile_with_options(
    fixture_path: &str,
    options: BundleOptions,
    plugins: Vec<Box<dyn Plugin>>,
  ) -> Bundler {
    compile_with_options_inner(fixture_path, options, plugins)
  }

  #[tokio::main]
  async fn compile_with_options_inner(
    fixture_path: &str,
    options: BundleOptions,
    plugins: Vec<Box<dyn Plugin>>,
  ) -> Bundler {
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
        ..options
      },
      plugins,
    );
    bundler.build().await;
    bundler.write_assets_to_disk();
    bundler
  }

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
      _path: &Path,
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
    compile("dynamic-import", vec![]);
  }

  #[test]
  #[ignore]
  fn basic_css() {
    let bundler = compile("basic-css", vec![]);
    assert!(bundler
      .plugin_driver
      .plugins
      .iter()
      .find(|plugin| plugin.name() == rspack_plugin_css::plugin::PLUGIN_NAME)
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
    compile("css", vec![]);
  }

  #[test]
  fn disable_code_splitting() {
    let bundler = compile_with_options(
      "basic",
      BundleOptions {
        code_splitting: false,
        ..Default::default()
      },
      vec![],
    );
    let chunk_len = bundler.ctx.assets.lock().unwrap().len();
    assert_eq!(chunk_len, 2);
  }

  #[test]
  fn enable_code_splitting() {
    let bundler = compile_with_options(
      "basic",
      BundleOptions {
        code_splitting: true,
        ..Default::default()
      },
      vec![],
    );
    let chunk_len = bundler.ctx.assets.lock().unwrap().len();
    assert_eq!(chunk_len, 4);
  }

  #[test]
  fn basic_ts() {
    compile("basic-ts", vec![]);
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
        loader: Some(
          vec![
            ("svg".to_string(), Loader::DataURI),
            // Json is supported by default
            // ("json".to_string(), Loader::Json),
            ("txt".to_string(), Loader::Text),
          ]
          .into_iter()
          .collect(),
        ),
        ..Default::default()
      },
      vec![],
    );
  }

  #[test]
  fn alias() {
    let bundler = compile_with_options(
      "alias",
      BundleOptions {
        resolve: ResolveOption {
          alias: vec![
            ("./wrong".to_string(), Some("./ok".to_string())),
            ("@/".to_string(), Some("./src/".to_string())),
          ],
          ..Default::default()
        },
        ..Default::default()
      },
      vec![],
    );
    let assets = bundler.ctx.assets.lock().unwrap();
    let dist = assets.get(0).unwrap();
    let source = &dist.source;
    println!("assets {:#?}", assets);
    assert!(!source.contains("wrong.js"));
    assert!(!source.contains("@"));
    assert!(source.contains("ok.js"));
    assert!(source.contains("at.js"));
  }

  #[test]

  fn stack_overflow_mockjs() {
    compile("stack_overflow_mockjs", vec![]);
  }
}
