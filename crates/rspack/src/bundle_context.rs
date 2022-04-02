use std::sync::{Mutex};

#[derive(Debug)]
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
