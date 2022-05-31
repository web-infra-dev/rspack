use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use anyhow::ensure;
use rspack_core::PluginTapGeneratedChunkHookOutput;

use crate::common::{compile_fixture_with_plugins, prelude::*};

#[derive(Debug, Default)]
struct Reocrd {
  filename_to_content: HashMap<String, String>,
  id_to_filename: HashMap<String, String>,
}

impl Reocrd {
  pub fn chunk_content_by_id(&self, id: &str) -> &str {
    let filename = self.id_to_filename[id].as_str();
    self.filename_to_content[filename].as_str()
  }
}

#[derive(Debug)]
struct Tester {
  record: Arc<Mutex<Reocrd>>,
}

impl Plugin for Tester {
  fn name(&self) -> &'static str {
    "Tester"
  }

  fn tap_generated_chunk(
    &self,
    _ctx: &rspack_core::BundleContext,
    chunk: &rspack_core::Chunk,
    _bundle_options: &rspack_core::NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    let mut record = self.record.lock().unwrap();
    record
      .id_to_filename
      .insert(chunk.id.clone(), chunk.filename.clone().unwrap());
    Ok(())
  }
}

#[tokio::test]
async fn code_splitting() -> anyhow::Result<()> {
  let record: Arc<Mutex<Reocrd>> = Default::default();
  let bundler = compile_fixture_with_plugins(
    "code-splitting",
    vec![Box::new(Tester {
      record: record.clone(),
    })],
  )
  .await;
  bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .iter()
    .for_each(|asset| {
      record
        .lock()
        .unwrap()
        .filename_to_content
        .insert(asset.filename.clone(), asset.source.clone());
    });
  let record = &record.lock().unwrap();
  ensure!(record.filename_to_content.len() == 2);
  ensure!(record
    .chunk_content_by_id("main")
    .contains("console.log('a')"));
  ensure!(record
    .chunk_content_by_id("main")
    .contains("console.log('b')"));
  ensure!(!record
    .chunk_content_by_id("src_c_js")
    .contains("console.log('b')"));
  ensure!(record
    .chunk_content_by_id("src_c_js")
    .contains("console.log('c')"));
  ensure!(record
    .chunk_content_by_id("src_c_js")
    .contains("console.log('d')"));
  Ok(())
}
