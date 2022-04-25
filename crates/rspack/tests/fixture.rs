mod testing {
  use async_trait::async_trait;
  use rspack::bundler::{BundleContext, BundleOptions, Bundler};
  use rspack::traits::plugin::{Plugin, ResolveHookOutput};
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env;
  use std::fs;
  use std::path::Path;
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
        outdir: Some(dist.to_str().unwrap().to_string()),
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
  struct TestPlugin {}
  #[async_trait]
  impl Plugin for TestPlugin {
    async fn resolve(
      &self,
      _ctx: &BundleContext,
      id: &str,
      importer: Option<&str>,
    ) -> ResolveHookOutput {
      println!("resolve:{:?},{:?}", id, importer);
      None
    }
  }

  #[test]
  fn plugin_test() {
    compile("single-entry", vec![Box::new(TestPlugin {})])
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
}
