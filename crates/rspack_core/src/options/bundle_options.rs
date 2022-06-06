use crate::{
  runtime::RuntimeOptions, BundleEntries, BundleMode, CodeSplittingOptions, LoaderOptions,
  OptimizationOptions, ResolveOption,
};
use std::collections::HashMap;

use rspack_swc::swc_ecma_transforms_react;

#[derive(Debug)]
pub struct BundleReactOptions {
  pub runtime: swc_ecma_transforms_react::Runtime,
  pub refresh: bool,
}

impl Default for BundleReactOptions {
  fn default() -> Self {
    Self {
      runtime: swc_ecma_transforms_react::Runtime::Automatic,
      refresh: false,
    }
  }
}

impl From<BundleMode> for BundleReactOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      runtime: Self::default().runtime,
      refresh: mode.is_dev(),
    }
  }
}

impl From<(BundleMode, Platform)> for BundleReactOptions {
  fn from((mode, platform): (BundleMode, Platform)) -> Self {
    Self {
      runtime: Self::default().runtime,
      refresh: mode.is_dev() && platform.is_browser(),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Platform {
  Browser,
  Node,
}

impl Default for Platform {
  fn default() -> Self {
    Self::Browser
  }
}

impl Platform {
  pub fn is_browser(&self) -> bool {
    matches!(self, Platform::Browser)
  }

  pub fn is_node(&self) -> bool {
    matches!(self, Platform::Node)
  }
}

#[derive(Debug)]
pub enum SourceMapOptions {
  None,
  Inline,
  External,
  Linked,
}

impl From<bool> for SourceMapOptions {
  fn from(option: bool) -> Self {
    if option {
      Self::Inline
    } else {
      Self::None
    }
  }
}

impl SourceMapOptions {
  pub fn is_enabled(&self) -> bool {
    !matches!(self, Self::None)
  }
}

#[derive(Debug)]
pub enum BundleTarget {
  CommonJS,
}

impl Default for BundleTarget {
  fn default() -> Self {
    Self::CommonJS
  }
}

#[derive(Debug)]
pub enum ChunkLoadingOptions {
  Import,
  JSONP,
}

impl Default for ChunkLoadingOptions {
  fn default() -> Self {
    Self::Import
  }
}

impl ChunkLoadingOptions {
  pub fn is_jsonp(&self) -> bool {
    matches!(self, ChunkLoadingOptions::JSONP)
  }

  pub fn is_import(&self) -> bool {
    matches!(self, ChunkLoadingOptions::Import)
  }
}

impl From<BundleMode> for ChunkLoadingOptions {
  fn from(mode: BundleMode) -> Self {
    if mode.is_prod() {
      Self::JSONP
    } else {
      Self::Import
    }
  }
}

#[derive(Debug)]
pub struct BundleOptions {
  pub platform: Platform,
  pub react: BundleReactOptions,
  pub loader: LoaderOptions,
  pub mode: BundleMode,
  pub entries: BundleEntries,
  pub minify: bool,
  pub outdir: String,
  pub entry_filename: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub chunk_filename: String,
  pub chunk_loading: ChunkLoadingOptions,
  pub code_splitting: CodeSplittingOptions,
  pub lazy_compilation: bool,
  pub root: String,
  pub inline_style: bool,
  pub resolve: ResolveOption,
  pub source_map: SourceMapOptions,
  pub svgr: bool,
  pub define: HashMap<String, String>,
  pub optimization: OptimizationOptions,
  pub progress: bool,
  pub globals: HashMap<String, String>,
  pub runtime: RuntimeOptions,
  pub target: BundleTarget,
}

impl BundleOptions {
  pub fn is_hmr_enabled(&self) -> bool {
    (self.runtime.hmr || self.react.refresh)
      && self.mode.is_dev()
      && self.platform == Platform::Browser
  }
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
      resolve: BundleMode::None.into(),
      react: Default::default(),
      root: std::env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string(),
      mode: BundleMode::Prod,
      entries: HashMap::from([("main".to_string(), "./src/index".to_string().into())]),
      // format: InternalModuleFormat::ES,
      outdir: std::env::current_dir()
        .unwrap()
        .join("./dist")
        .to_string_lossy()
        .to_string(),
      minify: Default::default(),
      entry_filename: "[name].js".to_string(),
      chunk_filename: "[id].js".to_string(),
      chunk_loading: Default::default(),
      code_splitting: Default::default(),
      lazy_compilation: false,
      loader: Default::default(),
      inline_style: Default::default(),
      source_map: true.into(),
      svgr: false,
      define: Default::default(),
      optimization: Default::default(),
      progress: true,
      globals: Default::default(),
      runtime: RuntimeOptions::default(),
      target: Default::default(),
      platform: Default::default(),
    }
  }
}

impl From<BundleMode> for BundleOptions {
  fn from(mode: BundleMode) -> Self {
    let BundleOptions {
      root,
      entry_filename,
      chunk_filename,
      loader,
      entries,
      outdir,
      inline_style,
      source_map,
      svgr,
      define,
      globals,
      ..
    } = Self::default();
    Self {
      platform: Platform::default(),
      mode,
      minify: mode.is_prod(),
      root,
      entry_filename,
      chunk_filename,
      chunk_loading: mode.into(),
      progress: !mode.is_none(),
      resolve: mode.into(),
      react: (mode, Platform::default()).into(),
      loader,
      entries,
      code_splitting: mode.into(),
      outdir,
      lazy_compilation: false,
      inline_style,
      // TODO: what's the right default value for different bundle mode
      source_map,
      svgr,
      define,
      globals,
      runtime: Default::default(),
      target: Default::default(),
      optimization: mode.into(),
    }
  }
}

impl From<(BundleMode, Platform)> for BundleOptions {
  fn from((mode, platform): (BundleMode, Platform)) -> Self {
    Self {
      react: (mode, platform).into(),
      ..mode.into()
    }
  }
}

pub type NormalizedBundleOptions = BundleOptions;
