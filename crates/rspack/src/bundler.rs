use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Context {
    assets: Mutex<Vec<Asset>>,
}

impl Context {
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
pub struct Bundler {
    ctx: Arc<Context>,
}

impl Bundler {
    pub fn write_assets_to_disk(&self) {
        self.ctx
            .assets
            .lock()
            .unwrap()
            .iter()
            .for_each(|asset| std::fs::write(&asset.filename, &asset.source).unwrap());
    }
}

#[derive(Debug)]
pub struct Asset {
    pub source: String,
    pub filename: String,
}
