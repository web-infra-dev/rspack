use std::{
  fmt::Debug,
  sync::{Arc, Mutex},
};

use swc::Compiler;

use crate::LoaderOptions;

#[derive(Debug, Clone)]
pub enum BundleMode {
  Dev,
  Prod,
  None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InternalModuleFormat {
  ES,
  CJS,
  AMD,
  UMD,
}

impl Default for InternalModuleFormat {
  fn default() -> Self {
    InternalModuleFormat::ES
  }
}

pub struct BundleContext {
  pub assets: Mutex<Vec<Asset>>,
  pub compiler: Arc<Compiler>,
  _noop: (),
}

impl Debug for BundleContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BundleContext")
      .field("assets", &self.assets)
      .field("compiler", &"{..}")
      .finish()
  }
}

impl BundleContext {
  pub fn new(compiler: Arc<Compiler>) -> Self {
    Self {
      assets: Default::default(),
      compiler,
      _noop: (),
    }
  }

  #[inline]
  pub fn emit_asset(&self, asset: Asset) {
    self.emit_assets([asset])
  }

  pub fn emit_assets(&self, assets_to_be_emited: impl IntoIterator<Item = Asset>) {
    let mut assets = self.assets.lock().unwrap();
    assets_to_be_emited.into_iter().for_each(|asset| {
      assets.push(asset);
    });
  }
}

#[derive(Debug)]
pub struct Asset {
  pub source: String,
  pub filename: String,
}

#[derive(Debug)]
pub struct BundleReactOptions {
  pub runtime: swc_ecma_transforms_react::Runtime,
}

impl Default for BundleReactOptions {
  fn default() -> Self {
    Self {
      runtime: swc_ecma_transforms_react::Runtime::Automatic,
    }
  }
}

#[derive(Debug)]
pub struct BundleOptions {
  pub react: BundleReactOptions,
  pub loader: Option<LoaderOptions>,
  pub mode: BundleMode,
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub outdir: String,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub code_splitting: bool,
  pub root: String,
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
      react: Default::default(),
      root: std::env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string(),
      mode: BundleMode::Prod,
      entries: Default::default(),
      // format: InternalModuleFormat::ES,
      outdir: std::env::current_dir()
        .unwrap()
        .join("./dist")
        .to_string_lossy()
        .to_string(),
      minify: Default::default(),
      entry_file_names: "[name].js".to_string(),
      code_splitting: true,
      loader: None,
    }
  }
}
