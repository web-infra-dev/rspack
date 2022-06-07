use std::{fs, path::Path};

use anyhow::ensure;

use crate::common::compile_fixture;

#[tokio::test]
async fn rebuild() -> anyhow::Result<()> {
  let mut bundler = compile_fixture("rebuild").await;
  ensure!(bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .unwrap()
    .source
    .contains("console.log('build')"));

  // change file
  let root = Path::new(&bundler.options.root);
  let entry = root.join("index.js").to_str().unwrap().to_string();
  fs::write(entry.clone(), "console.log('rebuild')")?;

  // rebuild
  bundler.rebuild(vec![entry.clone()]).await;
  ensure!(bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .unwrap()
    .source
    .contains("console.log('rebuild')"));

  // recover file
  fs::write(entry.clone(), "console.log('build')")?;

  Ok(())
}
