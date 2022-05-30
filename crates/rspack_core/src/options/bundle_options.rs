use crate::{
  runtime::RuntimeOptions, BundleEntries, BundleMode, CodeSplittingOptions, LoaderOptions,
  OptimizationOptions,
};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone)]
pub struct ResolveOption {
  pub extensions: Vec<String>,
  pub alias: Vec<(String, Option<String>)>,
  pub condition_names: HashSet<String>,
  pub symlinks: bool,
  pub alias_field: String,
}

impl Default for ResolveOption {
  fn default() -> Self {
    Self {
      extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
      alias: vec![],
      condition_names: Default::default(),
      symlinks: true,
      alias_field: String::from("browser"),
    }
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
  pub react: BundleReactOptions,
  pub loader: LoaderOptions,
  pub mode: BundleMode,
  pub entries: BundleEntries,
  pub minify: bool,
  pub outdir: String,
  pub entry_filename: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub chunk_filename: String,
  pub code_splitting: Option<CodeSplittingOptions>,
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
      entries: Default::default(),
      // format: InternalModuleFormat::ES,
      outdir: std::env::current_dir()
        .unwrap()
        .join("./dist")
        .to_string_lossy()
        .to_string(),
      minify: Default::default(),
      entry_filename: "[name].js".to_string(),
      chunk_filename: "[id].js".to_string(),
      code_splitting: Some(Default::default()),
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
    }
  }
}

pub type NormalizedBundleOptions = BundleOptions;
