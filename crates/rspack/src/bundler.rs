use std::sync::Arc;

use crate::{bundle_context::BundleContext, bundle_options::BundleOptions};

#[derive(Debug)]
pub struct Bundler {
    ctx: Arc<BundleContext>,
    options: Arc<BundleOptions>,
}

impl Bundler {
    pub fn new(options: BundleOptions) -> Self {
        Self {
            options: Arc::new(options),
            ctx: Default::default(),
        }
    }

    pub fn build() {

    }

    fn write_assets_to_disk(&self) {
        self.ctx
            .assets
            .lock()
            .unwrap()
            .iter()
            .for_each(|asset| std::fs::write(&asset.filename, &asset.source).unwrap());
    }
}
