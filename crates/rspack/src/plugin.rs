use std::{fmt::Debug, path::Path};

use crate::bundle_context::BundleContext;
use async_trait::async_trait;
use smol_str::SmolStr;
#[derive(Debug, Clone)]
pub struct ResolvedId {
    pub id: SmolStr,
    pub external: bool,
}

impl ResolvedId {
    pub fn new(id: String, external: bool) -> Self {
        Self {
            id: id.into(),
            external,
        }
    }
}

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
    #[inline]
    async fn prepare(&self, _ctx: &BundleContext) {}

    #[inline]
    async fn resolve(
        &self,
        _ctx: &BundleContext,
        _importer: Option<&str>,
        _mportee: &str,
    ) -> Option<ResolvedId> {
        None
    }

    #[inline]
    async fn load(&self, _ctx: &BundleContext, _id: &str) -> Option<String> {
        None
    }
}

#[derive(Debug)]

pub struct ResolveExtensionPlugin {
    pub extensions: Vec<String>,
}

#[async_trait]
impl Plugin for ResolveExtensionPlugin {
    async fn load(&self, _ctx: &BundleContext, id: &str) -> Option<String> {
        let p = Path::new(id);
        if p.extension().is_none() {
            let mut p = p.to_path_buf();
            for ext in &self.extensions {
                println!("check {:?}", p);
                p.set_extension(ext);
                let source = tokio::fs::read_to_string(&p).await;
                if let Ok(source) = source {
                    return Some(source);
                }
            }
            None
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PipedPlugin<A: Plugin, B: Plugin> {
    pub left: A,
    pub right: B,
}

impl <A: Plugin, B: Plugin> PipedPlugin<A, B> {
  pub fn new(left: A, right: B) -> Self {
    Self {
      left,
      right,
    }
  }
}

#[async_trait]
impl<A: Plugin, B: Plugin> Plugin for PipedPlugin<A, B> {
    async fn prepare(&self, ctx: &BundleContext) {
        tokio::join!(self.left.prepare(ctx), self.right.prepare(ctx));
    }

    async fn load(&self, ctx: &BundleContext, id: &str) -> Option<String> {
        self.left.load(ctx, id).await
    }

    async fn resolve(
        &self,
        ctx: &BundleContext,
        importer: Option<&str>,
        mportee: &str,
    ) -> Option<ResolvedId> {
        self.left.resolve(ctx, importer, mportee).await
    }
}
