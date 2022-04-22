use std::path::PathBuf;
use std::sync::Arc;

use std::sync::Mutex;

use sugar_path::PathSugar;

use crate::bundle::Bundle;
use crate::module_graph::ModuleGraph;
// use crate::module_graph_container::ModuleGraphContainer;
use crate::mark_box::MarkBox;
use crate::plugin_driver::PluginDriver;
use crate::structs::BundleMode;
use crate::traits::plugin::Plugin;

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
  pub outdir: Option<String>,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
}

impl Default for BundleOptions {
  fn default() -> Self {
    Self {
      mode: BundleMode::Prod,
      entries: Default::default(),
      // format: InternalModuleFormat::ES,
      outdir: Default::default(),
      minify: Default::default(),
      entry_file_names: "[name].js".to_string(),
    }
  }
}

#[derive(Debug)]
pub struct Bundler {
  pub ctx: Arc<BundleContext>,
  options: Arc<BundleOptions>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Bundler {
  pub fn new(options: BundleOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let ctx: Arc<BundleContext> = Default::default();
    Self {
      options: Arc::new(options),
      ctx: ctx.clone(),
      plugin_driver: Arc::new(PluginDriver {
        plugins,
        ctx: ctx.clone(),
      }),
    }
  }

  pub async fn generate(&mut self) {
    let mark_box: Arc<Mutex<MarkBox>> = Default::default();
    // let graph = ModuleGraphContainer::new(
    //     self.options.clone(),
    //     self.plugin_driver.clone(),
    //     mark_box.clone(),
    // );
    let module_graph =
      ModuleGraph::build_from(self.options.clone(), self.plugin_driver.clone()).await;

    let mut bundle = Bundle::new(module_graph, self.options.clone(), mark_box.clone());
    let output = bundle.generate();
    output.into_iter().for_each(|(_, chunk)| {
      self.ctx.assets.lock().unwrap().push(Asset {
        source: chunk.code,
        filename: chunk.file_name,
      })
    });
  }

  pub fn write_assets_to_disk(&self) {
    self.ctx.assets.lock().unwrap().iter().for_each(|asset| {
      let mut path = self
        .options
        .outdir
        .clone()
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| std::env::current_dir().unwrap());
      path.push(&asset.filename);
      std::fs::create_dir_all(path.resolve().parent().unwrap()).unwrap();
      std::fs::write(path.resolve(), &asset.source).unwrap();
    });
  }
}
