use std::sync::Mutex;

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

#[derive(Debug, Default)]
pub struct BundleContext {
  pub assets: Mutex<Vec<Asset>>,
}

impl BundleContext {
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
pub struct BundleOptions {
  pub mode: BundleMode,
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub outdir: String,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub code_splitting: bool,
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
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
    }
  }
}
