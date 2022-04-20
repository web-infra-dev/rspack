mod testing {
  use rspack::bundler::{BundleOptions, Bundler};
  use std::env;
  #[tokio::main]
  async fn compile(fixture_path: &str) {
    let dir = env::current_dir().unwrap();
    let entry = dir.join("fixtures").join(fixture_path).join("index.js");
    let dist = dir.join("fixtures").join(fixture_path).join("dist");
    println!("dir: {:?},{:?}", entry, dist);
    let mut bundler = Bundler::new(
      BundleOptions {
        entries: vec![entry.to_str().unwrap().to_string()],
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
}
