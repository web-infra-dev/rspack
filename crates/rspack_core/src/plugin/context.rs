use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};

use rspack_swc::swc::Compiler;

use crate::{Asset, NormalizedBundleOptions};

pub struct PluginContext<'me> {
  assets: &'me Arc<Mutex<Vec<Asset>>>,
  pub compiler: Arc<Compiler>,
  pub options: Arc<NormalizedBundleOptions>,
  pub(crate) resolved_entries: Arc<HashSet<String>>,
}

impl<'me> PluginContext<'me> {
  pub fn new(
    assets: &'me Arc<Mutex<Vec<Asset>>>,
    compiler: Arc<Compiler>,
    options: Arc<NormalizedBundleOptions>,
    resolved_entries: Arc<HashSet<String>>,
  ) -> Self {
    Self {
      assets,
      compiler,
      options,
      resolved_entries,
    }
  }

  pub fn is_entry_uri(&self, uri: &str) -> bool {
    self.resolved_entries.contains(uri)
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

  pub fn assets(&self) -> &'me Mutex<Vec<Asset>> {
    self.assets
  }
}
