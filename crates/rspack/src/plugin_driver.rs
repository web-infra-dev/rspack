use std::sync::Arc;

use crate::{bundler::BundleContext, structs::ResolvedId, traits::plugin::Plugin};

#[derive(Debug)]
pub struct PluginDriver {
    pub plugins: Vec<Box<dyn Plugin>>,
    pub ctx: Arc<BundleContext>,
}

impl PluginDriver {
    pub async fn resolve_id(&self, importee: &str, importer: Option<&str>) -> Option<ResolvedId> {
        for plugin in &self.plugins {
            let res = plugin.resolve(&self.ctx, importee, importer).await;
            if res.is_some() {
                return res;
            }
        }
        None
    }

    pub async fn load(&self, id: &str) -> Option<String> {
        for plugin in &self.plugins {
            let res = plugin.load(&self.ctx, id).await;
            if res.is_some() {
                return res;
            }
        }
        None
    }
}
