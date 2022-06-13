use std::{collections::HashMap, path::Path};

use rspack::bundler::{BundleOptions, Bundler};
use sugar_path::PathSugar;

#[tokio::main]
async fn main() {
  let entry_a_js = Path::new("./fixtures/basic/entry-a.js").resolve();
  let mut bundler = Bundler::new(
    BundleOptions {
      entries: HashMap::from([(
        "main".to_string(),
        "./fixtures/basic/entry-a.js".to_string().into(),
      )]),
      outdir: "./dist".to_string(),
      ..Default::default()
    },
    vec![],
  );
  std::fs::write(
    &entry_a_js,
    "
import { a } from './a'
// import { shared } from './shared'

console.log(a, shared)

import('./asynced').then(console.log)
  ",
  )
  .unwrap();
  bundler.build(None).await;
  std::fs::write(
    &entry_a_js,
    "
import { a } from './a'
import { shared } from './shared'

console.log(a, shared)

import('./asynced').then(console.log)
  ",
  )
  .unwrap();
  let rebuild_outout = bundler
    .rebuild(vec![entry_a_js.to_string_lossy().to_string()])
    .await;
  println!("rebuild_outout: {:#?}", rebuild_outout);
}
