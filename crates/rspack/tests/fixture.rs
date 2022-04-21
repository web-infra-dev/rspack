mod testing {
  use rspack::bundler::{BundleOptions, Bundler};
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env;
  use std::fs;
  use std::path::Path;
  #[tokio::main]
  async fn compile(fixture_path: &str) {
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
      vec![],
    );
    bundler.generate().await;
    bundler.write_assets_to_disk();
  }
  #[test]
  fn single_entry() {
    compile("single-entry")
  }
  #[test]
  fn multi_entry() {
    compile("multi-entry")
  }
}
