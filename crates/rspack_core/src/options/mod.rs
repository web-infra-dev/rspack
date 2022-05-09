use std::collections::HashMap;

#[derive(Debug)]
pub enum Loader {
  DataURI,
}

pub type LoaderOptions = HashMap<String, Loader>;

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

#[derive(Debug, Clone)]
pub enum BundleMode {
  Dev,
  Prod,
  None,
}

#[derive(Debug)]
pub struct BundleOptions {
  pub react: BundleReactOptions,
  pub loader: Option<LoaderOptions>,
  pub mode: BundleMode,
  pub entries: Vec<String>,
  pub minify: bool,
  pub outdir: String,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub code_splitting: bool,
  pub root: String,
  pub inline_style: bool,
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
      inline_style: Default::default(),
    }
  }
}

#[derive(Debug)]
pub struct DevServerOptions {
  pub hmr: bool,
}

impl Default for DevServerOptions {
  fn default() -> Self {
    Self { hmr: true }
  }
}

#[derive(Debug)]
pub struct NormalizedBundleOptions {
  pub react: BundleReactOptions,
  pub loader: Option<LoaderOptions>,
  pub mode: BundleMode,
  pub entries: Vec<String>,
  pub minify: bool,
  pub outdir: String,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub code_splitting: bool,
  pub root: String,
}
