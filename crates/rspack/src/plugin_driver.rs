use std::{path::Path, sync::Arc};

use crate::{bundler::BundleContext, plugin::Plugin, types::ResolvedId};

#[derive(Debug)]
pub struct PluginDriver {
    pub plugins: Vec<Box<dyn Plugin>>,
    pub ctx: Arc<BundleContext>,
}

#[inline]
pub fn is_external_module(source: &str) -> bool {
    source.starts_with("node:") || (!Path::new(source).is_absolute() && !source.starts_with('.'))
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
