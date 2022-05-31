use async_trait::async_trait;

use crate::common::{compile_fixture_with_plugins, prelude::*};
use rspack_core::{
  BundleContext, Chunk, LoadArgs, Loader, NormalizedBundleOptions, PluginLoadHookOutput,
  PluginResolveHookOutput, PluginTapGeneratedChunkHookOutput, PluginTransformAstHookOutput,
  PluginTransformHookOutput, ResolveArgs,
};
use rspack_swc::swc_ecma_ast;
use std::{
  path::Path,
  sync::{atomic::AtomicBool, Arc},
};

#[derive(Debug, Default)]
struct Record {
  call_resolve: Arc<AtomicBool>,
  call_load: Arc<AtomicBool>,
  call_transform: Arc<AtomicBool>,
  call_transform_ast: Arc<AtomicBool>,
  call_tap_generated_chunk: Arc<AtomicBool>,
}

#[derive(Debug)]
struct PluginHookTester {
  record: Arc<Record>,
}

#[async_trait]
impl Plugin for PluginHookTester {
  fn name(&self) -> &'static str {
    "rspack_test"
  }

  async fn resolve(&self, _ctx: &BundleContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    self
      .record
      .call_resolve
      .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(None)
  }

  async fn load(&self, _ctx: &BundleContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    self
      .record
      .call_load
      .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(None)
  }

  fn transform_ast(
    &self,
    _ctx: &BundleContext,
    _path: &Path,
    ast: swc_ecma_ast::Module,
  ) -> PluginTransformAstHookOutput {
    self
      .record
      .call_transform_ast
      .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(ast)
  }

  #[inline]
  fn transform(
    &self,
    _ctx: &BundleContext,
    _uri: &str,
    _loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    self
      .record
      .call_transform
      .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(raw)
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &BundleContext,
    _chunk: &Chunk,
    _bundle_options: &NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    self
      .record
      .call_tap_generated_chunk
      .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
  }
}

#[tokio::test]
async fn plugin_test() {
  let record: Arc<Record> = Default::default();
  assert!(!record.call_load.load(std::sync::atomic::Ordering::SeqCst));
  assert!(!record
    .call_resolve
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(!record
    .call_transform
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(!record
    .call_transform_ast
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(!record
    .call_tap_generated_chunk
    .load(std::sync::atomic::Ordering::SeqCst));
  let test_plugin = Box::new(PluginHookTester {
    record: record.clone(),
  });
  compile_fixture_with_plugins("plugin-hook", vec![test_plugin]).await;
  assert!(record.call_load.load(std::sync::atomic::Ordering::SeqCst));
  assert!(record
    .call_resolve
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(record
    .call_transform
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(record
    .call_transform_ast
    .load(std::sync::atomic::Ordering::SeqCst));
  assert!(record
    .call_tap_generated_chunk
    .load(std::sync::atomic::Ordering::SeqCst));
}
