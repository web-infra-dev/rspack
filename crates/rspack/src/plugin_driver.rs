use std::{path::Path, sync::Arc};

use sugar_path::PathSugar;

use crate::{
    bundle_context::BundleContext,
    plugin::{Plugin, ResolvedId},
};

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
    pub async fn resolve_id(&self, importer: Option<&str>, importee: &str) -> ResolvedId {
        let mut result = None;
        for plugin in &self.plugins {
            let res = plugin.resolve(&self.ctx, importer, importee).await;
            if res.is_some() {
                result = res;
            }
        }
        result.unwrap_or_else(|| {
            if importer.is_some() && is_external_module(importee) {
                ResolvedId::new(importee.to_string(), true)
            } else {
                let id = if let Some(importer) = importer {
                    Path::new(importer)
                        .parent()
                        .unwrap()
                        .join(importee)
                        .resolve()
                    // nodejs_path::resolve!(&nodejs_path::dirname(importer), source)
                } else {
                    Path::new(importee).resolve()
                };
                ResolvedId::new(id.to_string_lossy().to_string(), false)
            }
        })
    }

    pub async fn load(&self, id: &str) -> String {
        for plugin in &self.plugins {
            let res = plugin.load(&self.ctx, id).await;
            if res.is_some() {
                return res.unwrap();
            }
        }
        tokio::fs::read_to_string(id)
            .await
            .expect(&format!("{:?} is not exsit", id))
    }
}
