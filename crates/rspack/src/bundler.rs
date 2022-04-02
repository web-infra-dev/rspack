use std::sync::Arc;

use crate::bundle_context::BundleContext;


#[derive(Debug)]
pub struct Bundler {
    ctx: Arc<BundleContext>,
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

