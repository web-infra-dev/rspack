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

#[derive(Debug, PartialEq, Eq)]
pub enum Platform {
  Browser,
  Node,
}

impl Default for Platform {
  fn default() -> Self {
    Self::Browser
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
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
      resolve: Default::default(),
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
      platform: Default::default(),
    }
  }
}

impl From<BundleMode> for BundleOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      platform: Platform::default(),
      mode,
      minify: mode.is_prod(),
      root: Self::default().root,
      entry_filename: Self::default().entry_filename,
      chunk_filename: Self::default().chunk_filename,
      progress: !mode.is_none(),
      resolve: mode.into(),
      react: mode.into(),
      loader: Self::default().loader,
      entries: Self::default().entries,
      code_splitting: mode.into(),
      outdir: Self::default().outdir,
      lazy_compilation: false,
      inline_style: Self::default().inline_style,
      // TODO: what's the right default value for different bundle mode
      source_map: Self::default().source_map,
      svgr: Self::default().svgr,
      define: Self::default().define,
      globals: Self::default().globals,
      runtime: Default::default(),
      optimization: mode.into(),
    }
  }
}

pub type NormalizedBundleOptions = BundleOptions;
